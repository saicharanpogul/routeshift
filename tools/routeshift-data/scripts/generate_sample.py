#!/usr/bin/env python3
"""
Generate sample road network data for testing without osmnx.

Creates a realistic-looking grid+radial road network centered on each city,
with plausible road types, speeds, and capacities.
"""

import json
import math
import os
import random

random.seed(42)

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WEB_DATA_DIR = os.path.join(
    SCRIPT_DIR, "..", "..", "..", "apps", "routeshift-web", "public", "data"
)

CITIES = [
    {
        "name": "hyderabad",
        "center_lat": 17.385,
        "center_lon": 78.4867,
        "grid_size": 20,
        "spread": 0.04,  # degrees (~4.4km)
    },
    {
        "name": "mumbai",
        "center_lat": 19.076,
        "center_lon": 72.8777,
        "grid_size": 18,
        "spread": 0.035,
    },
    {
        "name": "bangalore",
        "center_lat": 12.9716,
        "center_lon": 77.5946,
        "grid_size": 20,
        "spread": 0.04,
    },
]

ROAD_TYPES = {
    "primary": {"speed_kmh": 50, "capacity_per_lane": 1500, "lanes": 2},
    "secondary": {"speed_kmh": 40, "capacity_per_lane": 1200, "lanes": 2},
    "tertiary": {"speed_kmh": 35, "capacity_per_lane": 1000, "lanes": 1},
    "residential": {"speed_kmh": 30, "capacity_per_lane": 800, "lanes": 1},
}


def haversine_km(lat1, lon1, lat2, lon2):
    R = 6371
    dlat = math.radians(lat2 - lat1)
    dlon = math.radians(lon2 - lon1)
    a = (
        math.sin(dlat / 2) ** 2
        + math.cos(math.radians(lat1))
        * math.cos(math.radians(lat2))
        * math.sin(dlon / 2) ** 2
    )
    return R * 2 * math.asin(math.sqrt(a))


def generate_city(city_config):
    name = city_config["name"]
    clat = city_config["center_lat"]
    clon = city_config["center_lon"]
    n = city_config["grid_size"]
    spread = city_config["spread"]

    nodes = []
    node_grid = {}

    # Create a grid of nodes with slight random perturbation
    node_id = 0
    for i in range(n):
        for j in range(n):
            lat = clat - spread + (2 * spread * i / (n - 1))
            lon = clon - spread + (2 * spread * j / (n - 1))
            # Add small random jitter for realism
            lat += random.uniform(-0.001, 0.001)
            lon += random.uniform(-0.001, 0.001)
            nodes.append({"id": node_id, "lat": round(lat, 6), "lon": round(lon, 6)})
            node_grid[(i, j)] = node_id
            node_id += 1

    edges = []
    edge_id = 0

    # Create grid edges (horizontal and vertical)
    for i in range(n):
        for j in range(n):
            src = node_grid[(i, j)]

            # Right neighbor
            if j < n - 1:
                tgt = node_grid[(i, j + 1)]
                edge_id = add_edge(nodes, edges, edge_id, src, tgt, i, j, n)

            # Down neighbor
            if i < n - 1:
                tgt = node_grid[(i + 1, j)]
                edge_id = add_edge(nodes, edges, edge_id, src, tgt, i, j, n)

            # Reverse edges (bidirectional roads)
            # Left neighbor
            if j > 0:
                tgt = node_grid[(i, j - 1)]
                edge_id = add_edge(nodes, edges, edge_id, src, tgt, i, j, n)

            # Up neighbor
            if i > 0:
                tgt = node_grid[(i - 1, j)]
                edge_id = add_edge(nodes, edges, edge_id, src, tgt, i, j, n)

            # Some diagonal connections for realism (arterial roads)
            if i < n - 1 and j < n - 1 and random.random() < 0.15:
                tgt = node_grid[(i + 1, j + 1)]
                edge_id = add_edge(
                    nodes, edges, edge_id, src, tgt, i, j, n, diagonal=True
                )

    # Generate OD pairs
    od_pairs = generate_od_pairs(nodes, n, num_pairs=300)

    # Build graph JSON
    graph_data = {"nodes": nodes, "edges": edges, "od_pairs": od_pairs}

    # Build GeoJSON
    features = []
    for edge in edges:
        src_node = nodes[edge["source"]]
        tgt_node = nodes[edge["target"]]
        features.append(
            {
                "type": "Feature",
                "properties": {
                    "edge_id": edge["id"],
                    "road_type": edge.get("road_type", "residential"),
                    "name": "",
                    "lanes": edge.get("lanes", 1),
                },
                "geometry": {
                    "type": "LineString",
                    "coordinates": [
                        [src_node["lon"], src_node["lat"]],
                        [tgt_node["lon"], tgt_node["lat"]],
                    ],
                },
            }
        )

    geojson = {"type": "FeatureCollection", "features": features}

    # Write files
    os.makedirs(WEB_DATA_DIR, exist_ok=True)

    graph_path = os.path.join(WEB_DATA_DIR, f"{name}_graph.json")
    with open(graph_path, "w") as f:
        json.dump(graph_data, f, separators=(",", ":"))

    geojson_path = os.path.join(WEB_DATA_DIR, f"{name}.geojson")
    with open(geojson_path, "w") as f:
        json.dump(geojson, f, separators=(",", ":"))

    print(
        f"{name}: {len(nodes)} nodes, {len(edges)} edges, {len(od_pairs)} OD pairs"
    )
    print(f"  Graph: {graph_path}")
    print(f"  GeoJSON: {geojson_path}")


def add_edge(nodes, edges, edge_id, src, tgt, i, j, n, diagonal=False):
    src_node = nodes[src]
    tgt_node = nodes[tgt]
    length_km = haversine_km(
        src_node["lat"], src_node["lon"], tgt_node["lat"], tgt_node["lon"]
    )

    # Determine road type based on position in grid
    dist_from_center = math.sqrt((i - n / 2) ** 2 + (j - n / 2) ** 2) / (n / 2)

    if dist_from_center < 0.3:
        road_type = "primary"
    elif dist_from_center < 0.6:
        road_type = "secondary"
    elif diagonal:
        road_type = "secondary"
    else:
        road_type = random.choice(["tertiary", "residential"])

    road_info = ROAD_TYPES[road_type]
    speed_kmh = road_info["speed_kmh"] * random.uniform(0.8, 1.2)
    free_flow_time = (length_km / speed_kmh) * 60  # minutes
    capacity = road_info["capacity_per_lane"] * road_info["lanes"]

    edges.append(
        {
            "id": edge_id,
            "source": src,
            "target": tgt,
            "free_flow_time": round(max(free_flow_time, 0.01), 4),
            "capacity": float(capacity),
            "length_km": round(length_km, 4),
            "road_type": road_type,
            "lanes": road_info["lanes"],
        }
    )
    return edge_id + 1


def generate_od_pairs(nodes, grid_size, num_pairs=300):
    pairs = []
    n = len(nodes)
    seen = set()

    for _ in range(num_pairs * 5):
        if len(pairs) >= num_pairs:
            break

        # Bias origins toward residential areas (edges of grid) and
        # destinations toward center (commercial areas)
        origin = random.randint(0, n - 1)
        dest = random.randint(0, n - 1)

        if origin == dest:
            continue
        key = (origin, dest)
        if key in seen:
            continue

        # Skip very short trips
        o = nodes[origin]
        d = nodes[dest]
        dist = haversine_km(o["lat"], o["lon"], d["lat"], d["lon"])
        if dist < 0.5:
            continue

        seen.add(key)
        pairs.append(
            {
                "origin": origin,
                "destination": dest,
                "demand": round(random.uniform(1.0, 8.0), 1),
            }
        )

    return pairs


if __name__ == "__main__":
    print("Generating sample road network data...")
    print(f"Output: {WEB_DATA_DIR}\n")

    for city in CITIES:
        generate_city(city)

    print("\nDone! Data ready for the frontend.")
