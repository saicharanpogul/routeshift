#!/usr/bin/env python3
"""
Extract road networks from OpenStreetMap for RouteShift cities.

Uses osmnx v2 API. Requires: pip install osmnx geopandas networkx shapely

Usage: python extract_network.py [--city hyderabad|mumbai|bangalore|all]
"""

import argparse
import json
import os
import random

import networkx as nx
import osmnx as ox

random.seed(42)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_DIR = os.path.join(SCRIPT_DIR, "..", "output")
WEB_DATA_DIR = os.path.join(
    SCRIPT_DIR, "..", "..", "..", "apps", "routeshift-web", "public", "data"
)

CITIES = [
    {
        "name": "hyderabad",
        "center": (17.440, 78.498),  # HITEC City area
        "dist": 3000,  # 3km radius
    },
    {
        "name": "mumbai",
        "center": (19.076, 72.878),  # Bandra-Kurla area
        "dist": 3000,
    },
    {
        "name": "bangalore",
        "center": (12.972, 77.595),  # MG Road area
        "dist": 3000,
    },
]

# Default speed limits (km/h) by road type
SPEED_DEFAULTS = {
    "motorway": 80, "motorway_link": 60,
    "trunk": 60, "trunk_link": 50,
    "primary": 50, "primary_link": 40,
    "secondary": 40, "secondary_link": 35,
    "tertiary": 35, "tertiary_link": 30,
    "residential": 25, "living_street": 15,
    "unclassified": 25, "service": 15,
}

# Capacity per lane (vehicles/hour) by road type
CAPACITY_PER_LANE = {
    "motorway": 2000, "motorway_link": 1500,
    "trunk": 1800, "trunk_link": 1200,
    "primary": 1500, "primary_link": 1000,
    "secondary": 1200, "secondary_link": 800,
    "tertiary": 1000, "tertiary_link": 600,
    "residential": 600, "living_street": 300,
    "unclassified": 500, "service": 300,
}

DEFAULT_LANES = {
    "motorway": 3, "motorway_link": 2,
    "trunk": 2, "trunk_link": 1,
    "primary": 2, "primary_link": 1,
    "secondary": 2, "secondary_link": 1,
    "tertiary": 1, "tertiary_link": 1,
    "residential": 1, "living_street": 1,
    "unclassified": 1, "service": 1,
}


def first_of(val, default="residential"):
    """Get first value if list, else return as-is."""
    if isinstance(val, list):
        return val[0] if val else default
    return val if val else default


def get_speed(data):
    maxspeed = data.get("maxspeed")
    if maxspeed:
        ms = first_of(maxspeed)
        try:
            return float(str(ms).split()[0])
        except (ValueError, TypeError):
            pass
    highway = first_of(data.get("highway"))
    return SPEED_DEFAULTS.get(highway, 25)


def get_capacity(data):
    highway = first_of(data.get("highway"))
    lanes_raw = data.get("lanes")
    if lanes_raw:
        try:
            lanes = int(first_of(lanes_raw))
        except (ValueError, TypeError):
            lanes = DEFAULT_LANES.get(highway, 1)
    else:
        lanes = DEFAULT_LANES.get(highway, 1)
    return lanes * CAPACITY_PER_LANE.get(highway, 500)


def extract_city(city_cfg):
    name = city_cfg["name"]
    center = city_cfg["center"]
    dist = city_cfg["dist"]

    print(f"\nDownloading {name} road network (radius={dist}m)...")
    G = ox.graph_from_point(center, dist=dist, network_type="drive", simplify=True)

    print(f"  Raw: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")

    # Re-index nodes to contiguous 0-based IDs
    node_list = list(G.nodes())
    node_map = {old: new for new, old in enumerate(node_list)}
    G = nx.relabel_nodes(G, node_map)

    print(f"  Re-indexed: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges")
    return G


def export_data(G, city_name):
    os.makedirs(WEB_DATA_DIR, exist_ok=True)
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    # Build nodes
    nodes = []
    for nid, data in G.nodes(data=True):
        nodes.append({
            "id": int(nid),
            "lat": float(data["y"]),
            "lon": float(data["x"]),
        })

    # Build edges + GeoJSON features
    edges = []
    features = []
    edge_id = 0

    for u, v, data in G.edges(data=True):
        length_m = data.get("length", 100)
        length_km = length_m / 1000.0
        speed = get_speed(data)
        free_flow_time = (length_km / speed) * 60.0 if speed > 0 else 1.0  # minutes
        capacity = get_capacity(data)
        highway = first_of(data.get("highway"))
        road_name = first_of(data.get("name", ""))

        edges.append({
            "id": edge_id,
            "source": int(u),
            "target": int(v),
            "free_flow_time": round(max(free_flow_time, 0.001), 4),
            "capacity": float(capacity),
            "length_km": round(length_km, 4),
        })

        # Geometry: use actual road geometry if available
        geom = data.get("geometry")
        if geom:
            coords = [[round(c[0], 6), round(c[1], 6)] for c in geom.coords]
        else:
            u_data = G.nodes[u]
            v_data = G.nodes[v]
            coords = [
                [round(float(u_data["x"]), 6), round(float(u_data["y"]), 6)],
                [round(float(v_data["x"]), 6), round(float(v_data["y"]), 6)],
            ]

        features.append({
            "type": "Feature",
            "properties": {
                "edge_id": edge_id,
                "road_type": highway,
                "name": str(road_name) if road_name else "",
                "lanes": DEFAULT_LANES.get(highway, 1),
                "capacity": float(capacity),
            },
            "geometry": {
                "type": "LineString",
                "coordinates": coords,
            },
        })
        edge_id += 1

    # Generate OD pairs
    od_pairs = generate_od_pairs(G, 400)

    # Write graph JSON
    graph_data = {"nodes": nodes, "edges": edges, "od_pairs": od_pairs}
    graph_path = os.path.join(WEB_DATA_DIR, f"{city_name}_graph.json")
    with open(graph_path, "w") as f:
        json.dump(graph_data, f, separators=(",", ":"))

    # Write GeoJSON
    geojson = {"type": "FeatureCollection", "features": features}
    geojson_path = os.path.join(WEB_DATA_DIR, f"{city_name}.geojson")
    with open(geojson_path, "w") as f:
        json.dump(geojson, f, separators=(",", ":"))

    print(f"  Exported: {len(nodes)} nodes, {len(edges)} edges, {len(od_pairs)} OD pairs")
    print(f"  Graph: {graph_path}")
    print(f"  GeoJSON: {geojson_path}")


def generate_od_pairs(G, num_pairs=400):
    nodes = list(G.nodes())
    if len(nodes) < 2:
        return []

    degrees = dict(G.degree())
    total_deg = sum(degrees.values()) or 1
    weights = [degrees.get(n, 1) / total_deg for n in nodes]

    pairs = []
    seen = set()
    for _ in range(num_pairs * 10):
        if len(pairs) >= num_pairs:
            break
        o = random.choices(nodes, weights=weights, k=1)[0]
        d = random.choices(nodes, weights=weights, k=1)[0]
        if o == d or (o, d) in seen:
            continue
        if nx.has_path(G, o, d):
            seen.add((o, d))
            pairs.append({
                "origin": int(o),
                "destination": int(d),
                "demand": round(random.uniform(1.0, 8.0), 1),
            })

    print(f"  Generated {len(pairs)} OD pairs")
    return pairs


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--city", choices=["hyderabad", "mumbai", "bangalore", "all"], default="all")
    args = parser.parse_args()

    cities = CITIES if args.city == "all" else [c for c in CITIES if c["name"] == args.city]

    for city in cities:
        G = extract_city(city)
        export_data(G, city["name"])

    print("\nDone!")


if __name__ == "__main__":
    main()
