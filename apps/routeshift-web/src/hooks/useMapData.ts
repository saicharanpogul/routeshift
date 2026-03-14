"use client";

import { useState, useEffect } from "react";
import { RoadNetwork } from "@/types/graph";

interface MapData {
  geojson: GeoJSON.FeatureCollection | null;
  graph: RoadNetwork | null;
  loading: boolean;
  error: string | null;
}

export function useMapData(cityName: string): MapData {
  const [geojson, setGeojson] = useState<GeoJSON.FeatureCollection | null>(
    null
  );
  const [graph, setGraph] = useState<RoadNetwork | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    async function loadData() {
      try {
        const [geojsonRes, graphRes] = await Promise.all([
          fetch(`/data/${cityName}.geojson`),
          fetch(`/data/${cityName}_graph.json`),
        ]);

        if (!geojsonRes.ok)
          throw new Error(`Failed to load GeoJSON for ${cityName}`);
        if (!graphRes.ok)
          throw new Error(`Failed to load graph for ${cityName}`);

        const [geojsonData, graphData] = await Promise.all([
          geojsonRes.json(),
          graphRes.json(),
        ]);

        if (!cancelled) {
          setGeojson(geojsonData);
          setGraph(graphData);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof Error ? err.message : "Failed to load map data"
          );
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    loadData();
    return () => {
      cancelled = true;
    };
  }, [cityName]);

  return { geojson, graph, loading, error };
}
