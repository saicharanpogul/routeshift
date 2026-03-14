import { Node } from "@/types/graph";

/**
 * Extract edge geometries from GeoJSON for passing to WASM.
 * Returns array indexed by edge_id, each containing [[lng, lat], ...]
 */
export function extractEdgeGeometries(
  geojson: GeoJSON.FeatureCollection
): number[][][] {
  const geometries: number[][][] = [];

  for (const feature of geojson.features) {
    const edgeId = feature.properties?.edge_id ?? geometries.length;
    const coords = (feature.geometry as GeoJSON.LineString).coordinates;
    // Ensure array is large enough
    while (geometries.length <= edgeId) {
      geometries.push([]);
    }
    geometries[edgeId] = coords.map((c) => [c[0], c[1]]);
  }

  return geometries;
}

/**
 * Find the nearest graph node to a click point.
 */
export function findNearestNode(
  lngLat: [number, number],
  nodes: Node[]
): { nodeId: number; distance: number } {
  let bestId = 0;
  let bestDist = Infinity;

  for (const node of nodes) {
    const dlng = node.lon - lngLat[0];
    const dlat = node.lat - lngLat[1];
    const dist = dlng * dlng + dlat * dlat;
    if (dist < bestDist) {
      bestDist = dist;
      bestId = node.id;
    }
  }

  return { nodeId: bestId, distance: Math.sqrt(bestDist) };
}

/**
 * Build a GeoJSON FeatureCollection of Points from flat car position array.
 */
export function buildCarGeoJSON(
  positions: number[],
  types: number[]
): GeoJSON.FeatureCollection {
  const features: GeoJSON.Feature[] = [];
  const numCars = types.length;

  for (let i = 0; i < numCars; i++) {
    const lng = positions[i * 2];
    const lat = positions[i * 2 + 1];
    if (lng === 0 && lat === 0) continue;

    features.push({
      type: "Feature",
      properties: { carType: types[i] },
      geometry: {
        type: "Point",
        coordinates: [lng, lat],
      },
    });
  }

  return { type: "FeatureCollection", features };
}

/**
 * Build a GeoJSON LineString from route geometry coordinates.
 */
export function buildRouteLineString(
  geometry: [number, number][]
): GeoJSON.Feature {
  return {
    type: "Feature",
    properties: {},
    geometry: {
      type: "LineString",
      coordinates: geometry,
    },
  };
}
