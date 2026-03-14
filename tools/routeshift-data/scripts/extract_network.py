#!/usr/bin/env python3
"""
Extract road networks from OpenStreetMap for RouteShift cities.

Requires: pip install osmnx geopandas networkx shapely

Usage: python extract_network.py [--city hyderabad|mumbai|bangalore|all]
"""

import argparse
import json
import os
import sys

import geopandas as gpd
import networkx as nx
import osmnx as ox

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
CONFIG_PATH = os.path.join(SCRIPT_DIR, "..", "config", "cities.json")
OUTPUT_DIR = os.path.join(SCRIPT_DIR, "..", "output")
WEB_DATA_DIR = os.path.join(
    SCRIPT_DIR, "..", "..", "..", "apps", "routeshift-web", "public", "data"
)

# Default speed limits (km/h) by road type when OSM data is missing
SPEED_DEFAULTS = {
    "motorway": 80,
    "motorway_link": 60,
    "trunk": 60,
    "trunk_link": 50,
    "primary": 50,
    "primary_link": 40,
    "secondary": 40,
    "secondary_link": 35,
    "tertiary": 35,
    "tertiary_link": 30,
    "residential": 30,
    "living_street": 20,
    "unclassified": 30,
    "service": 20,
}

# Capacity per lane (vehicles/hour) by road type
CAPACITY_PER_LANE = {
    "motorway": 2000,
    "motorway_link": 1500,
    "trunk": 1800,
    "trunk_link": 1200,
    "primary": 1500,
    "primary_link": 1000,
    "secondary": 1200,
    "secondary_link": 800,
    "tertiary": 1000,
    "tertiary_link": 600,
    "residential": 800,
    "living_street": 400,
    "unclassified": 600,
    "service": 400,
}

# Default lanes by road type
DEFAULT_LANES = {
    "motorway": 3,
    "motorway_link": 2,
    "trunk": 2,
    "trunk_link": 1,
    "primary": 2,
    "primary_link": 1,
    "secondary": 2,
    "secondary_link": 1,
    "tertiary": 1,
    "tertiary_link": 1,
    "residential": 1,
    "living_street": 1,
    "unclassified": 1,
    "service": 1,
}


def load_config():
    with open(CONFIG_PATH) as f:
        return json.load(f)


def get_speed(edge_data):
    """Extract speed limit from OSM data or use defaults."""
    maxspeed = edge_data.get("maxspeed")
    if maxspeed:
        if isinstance(maxspeed, list):
            maxspeed = maxspeed[0]
        try:
            return float(str(maxspeed).replace(" km/h", "").replace(" mph", ""))
        except (ValueError, TypeError):
            pass

    highway = edge_data.get("highway", "residential")
    if isinstance(highway, list):
        highway = highway[0]
    return SPEED_DEFAULTS.get(highway, 30)


def get_capacity(edge_data):
    """Estimate road capacity from lanes and road type."""
    highway = edge_data.get("highway", "residential")
    if isinstance(highway, list):
        highway = highway[0]

    lanes = edge_data.get("lanes")
    if lanes:
        if isinstance(lanes, list):
            lanes = lanes[0]
        try:
            lanes = int(lanes)
        except (ValueError, TypeError):
            lanes = DEFAULT_LANES.get(highway, 1)
    else:
        lanes = DEFAULT_LANES.get(highway, 1)

    cap_per_lane = CAPACITY_PER_LANE.get(highway, 600)
    return lanes * cap_per_lane


def extract_city(city_config):
    """Download and process road network for a city."""
    name = city_config["name"]
    center = city_config["center"]
    radius_m = city_config["radius_km"] * 1000

    print(f"Downloading road network for {city_config['label']}...")
    G = ox.graph_from_point(
        (center[0], center[1]),
        dist=radius_m,
        network_type=city_config["network_type"],
        simplify=True,
    )

    # Consolidate intersections for cleaner graph
    G = ox.consolidate_intersections(ox.project_graph(G), tolerance=15, rebuild_graph=True)
    G = ox.project_graph(G, to_crs="EPSG:4326")

    print(f"  Nodes: {G.number_of_nodes()}, Edges: {G.number_of_edges()}")

    # Re-index nodes to contiguous 0-based IDs
    node_mapping = {old_id: new_id for new_id, old_id in enumerate(G.nodes())}
    G = nx.relabel_nodes(G, node_mapping)

    return G, node_mapping


def export_graph_json(G, city_name, output_dir):
    """Export graph as compact JSON for WASM consumption."""
    nodes = []
    for node_id, data in G.nodes(data=True):
        nodes.append({
            "id": int(node_id),
            "lat": float(data.get("y", data.get("lat", 0))),
            "lon": float(data.get("x", data.get("lon", 0))),
        })

    edges = []
    edge_id = 0
    for u, v, data in G.edges(data=True):
        length_m = data.get("length", 100)
        length_km = length_m / 1000.0
        speed_kmh = get_speed(data)
        free_flow_time = (length_km / speed_kmh) * 60.0  # minutes

        edges.append({
            "id": edge_id,
            "source": int(u),
            "target": int(v),
            "free_flow_time": round(free_flow_time, 4),
            "capacity": float(get_capacity(data)),
            "length_km": round(length_km, 4),
        })
        edge_id += 1

    # Generate synthetic OD pairs
    od_pairs = generate_od_pairs(G, num_pairs=500)

    graph_data = {
        "nodes": nodes,
        "edges": edges,
        "od_pairs": od_pairs,
    }

    os.makedirs(output_dir, exist_ok=True)
    output_path = os.path.join(output_dir, f"{city_name}_graph.json")
    with open(output_path, "w") as f:
        json.dump(graph_data, f, separators=(",", ":"))
    print(f"  Graph JSON: {output_path} ({len(nodes)} nodes, {len(edges)} edges)")
    return graph_data


def export_geojson(G, city_name, output_dir):
    """Export road network as GeoJSON for MapLibre rendering."""
    features = []
    edge_id = 0

    for u, v, data in G.edges(data=True):
        geometry = data.get("geometry")
        if geometry:
            coords = list(geometry.coords)
        else:
            u_data = G.nodes[u]
            v_data = G.nodes[v]
            coords = [
                [float(u_data.get("x", u_data.get("lon", 0))),
                 float(u_data.get("y", u_data.get("lat", 0)))],
                [float(v_data.get("x", v_data.get("lon", 0))),
                 float(v_data.get("y", v_data.get("lat", 0)))],
            ]

        # Ensure coords are [lon, lat] format
        formatted_coords = []
        for coord in coords:
            if len(coord) >= 2:
                formatted_coords.append([round(float(coord[0]), 6), round(float(coord[1]), 6)])

        highway = data.get("highway", "unclassified")
        if isinstance(highway, list):
            highway = highway[0]

        name = data.get("name", "")
        if isinstance(name, list):
            name = name[0]

        feature = {
            "type": "Feature",
            "properties": {
                "edge_id": edge_id,
                "road_type": highway,
                "name": str(name) if name else "",
                "lanes": int(data.get("lanes", DEFAULT_LANES.get(highway, 1)))
                if not isinstance(data.get("lanes"), list)
                else int(data["lanes"][0]),
            },
            "geometry": {
                "type": "LineString",
                "coordinates": formatted_coords,
            },
        }
        features.append(feature)
        edge_id += 1

    geojson = {
        "type": "FeatureCollection",
        "features": features,
    }

    os.makedirs(output_dir, exist_ok=True)
    output_path = os.path.join(output_dir, f"{city_name}.geojson")
    with open(output_path, "w") as f:
        json.dump(geojson, f, separators=(",", ":"))
    print(f"  GeoJSON: {output_path} ({len(features)} features)")


def generate_od_pairs(G, num_pairs=500):
    """Generate synthetic OD demand pairs weighted by node degree."""
    import random
    random.seed(42)

    nodes = list(G.nodes())
    if len(nodes) < 2:
        return []

    # Weight by degree (higher degree = more likely origin/destination)
    degrees = dict(G.degree())
    total_degree = sum(degrees.values())
    weights = [degrees[n] / total_degree for n in nodes]

    od_pairs = []
    seen = set()
    attempts = 0
    max_attempts = num_pairs * 10

    while len(od_pairs) < num_pairs and attempts < max_attempts:
        attempts += 1
        origin = random.choices(nodes, weights=weights, k=1)[0]
        dest = random.choices(nodes, weights=weights, k=1)[0]

        if origin == dest:
            continue
        pair_key = (origin, dest)
        if pair_key in seen:
            continue

        # Check connectivity
        if nx.has_path(G, origin, dest):
            seen.add(pair_key)
            od_pairs.append({
                "origin": int(origin),
                "destination": int(dest),
                "demand": round(random.uniform(1.0, 10.0), 1),
            })

    print(f"  Generated {len(od_pairs)} OD pairs")
    return od_pairs


def main():
    parser = argparse.ArgumentParser(description="Extract road networks from OSM")
    parser.add_argument(
        "--city",
        choices=["hyderabad", "mumbai", "bangalore", "all"],
        default="all",
        help="City to extract (default: all)",
    )
    args = parser.parse_args()

    config = load_config()
    cities = config["cities"]

    if args.city != "all":
        cities = [c for c in cities if c["name"] == args.city]

    os.makedirs(OUTPUT_DIR, exist_ok=True)
    os.makedirs(WEB_DATA_DIR, exist_ok=True)

    for city in cities:
        print(f"\n{'='*60}")
        print(f"Processing {city['label']}")
        print(f"{'='*60}")

        G, _ = extract_city(city)
        export_graph_json(G, city["name"], OUTPUT_DIR)
        export_geojson(G, city["name"], WEB_DATA_DIR)

        # Also copy graph JSON to web data dir for frontend access
        import shutil
        src = os.path.join(OUTPUT_DIR, f"{city['name']}_graph.json")
        dst = os.path.join(WEB_DATA_DIR, f"{city['name']}_graph.json")
        shutil.copy2(src, dst)

    print("\nDone!")


if __name__ == "__main__":
    main()
