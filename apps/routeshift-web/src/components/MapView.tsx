"use client";

import { useRef, useEffect, useCallback } from "react";
import {
  Map,
  Source,
  Layer,
  NavigationControl,
  type MapRef,
} from "@vis.gl/react-maplibre";
import type { LayerSpecification } from "maplibre-gl";
import { CityConfig } from "@/types/graph";
import { RouteOption } from "@/types/game";
import { useGameStore } from "@/stores/gameStore";
import { buildCarGeoJSON } from "@/lib/geometryUtils";

const MAP_STYLE = "https://tiles.openfreemap.org/styles/liberty";

const EMPTY_FC: GeoJSON.FeatureCollection = {
  type: "FeatureCollection",
  features: [],
};

interface MapViewProps {
  city: CityConfig;
  geojson: GeoJSON.FeatureCollection | null;
  onMapClick: (lngLat: [number, number]) => void;
  routeOptions: RouteOption[] | null;
  hoveredRouteIndex: number | null;
  clickable: boolean;
}

export function MapView({
  city,
  geojson,
  onMapClick,
  routeOptions,
  hoveredRouteIndex,
  clickable,
}: MapViewProps) {
  const mapRef = useRef<MapRef>(null);
  const prevCityRef = useRef(city.name);

  // Fly to new city when changed
  useEffect(() => {
    if (prevCityRef.current !== city.name && mapRef.current) {
      mapRef.current.flyTo({
        center: [city.center[1], city.center[0]],
        zoom: city.zoom,
        duration: 1500,
      });
      prevCityRef.current = city.name;
    }
  }, [city]);

  // Imperative car position updates from Zustand (outside React render cycle)
  useEffect(() => {
    const unsub = useGameStore.subscribe((state) => {
      const map = mapRef.current?.getMap();
      if (!map || !map.isStyleLoaded()) return;

      const { carPositions, carTypes } = state;
      if (!carPositions || !carTypes) return;

      // Build separate GeoJSON for AI cars and player car
      const aiFeatures: GeoJSON.Feature[] = [];
      let playerFeature: GeoJSON.Feature | null = null;

      const numCars = carTypes.length;
      for (let i = 0; i < numCars; i++) {
        const lng = carPositions[i * 2];
        const lat = carPositions[i * 2 + 1];
        if (lng === 0 && lat === 0) continue;

        const feature: GeoJSON.Feature = {
          type: "Feature",
          properties: {},
          geometry: { type: "Point", coordinates: [lng, lat] },
        };

        if (carTypes[i] === 1) {
          playerFeature = feature;
        } else {
          aiFeatures.push(feature);
        }
      }

      const aiSource = map.getSource("ai-cars") as maplibregl.GeoJSONSource;
      if (aiSource) {
        aiSource.setData({ type: "FeatureCollection", features: aiFeatures });
      }

      const playerSource = map.getSource("player-car") as maplibregl.GeoJSONSource;
      if (playerSource) {
        playerSource.setData({
          type: "FeatureCollection",
          features: playerFeature ? [playerFeature] : [],
        });
      }

      // Update congestion coloring (throttled by rAF)
      if (state.edgeFlows && geojson) {
        const roadSource = map.getSource("road-network") as maplibregl.GeoJSONSource;
        if (roadSource) {
          const updatedFeatures = geojson.features.map((f, idx) => {
            const edgeId = f.properties?.edge_id ?? idx;
            const flow = state.edgeFlows![edgeId] ?? 0;
            const capacity = (f.properties?.capacity as number) ?? 1000;
            const vcRatio = capacity > 0 ? flow / capacity : 0;
            return {
              ...f,
              properties: { ...f.properties, vc_ratio: vcRatio, flow },
            };
          });
          roadSource.setData({ ...geojson, features: updatedFeatures });
        }
      }
    });

    return unsub;
  }, [geojson]);

  // Build route options GeoJSON
  const routeGeoJSON: GeoJSON.FeatureCollection = routeOptions
    ? {
        type: "FeatureCollection",
        features: routeOptions.map((route, i) => ({
          type: "Feature" as const,
          properties: {
            index: i,
            routeType: route.route_type,
            active: hoveredRouteIndex === i ? 1 : 0,
          },
          geometry: {
            type: "LineString" as const,
            coordinates: route.geometry,
          },
        })),
      }
    : EMPTY_FC;

  const handleClick = useCallback(
    (e: maplibregl.MapMouseEvent) => {
      if (clickable) {
        onMapClick([e.lngLat.lng, e.lngLat.lat]);
      }
    },
    [clickable, onMapClick]
  );

  const networkLayer: LayerSpecification = {
    id: "road-network",
    type: "line",
    source: "road-network",
    paint: {
      "line-color": [
        "interpolate",
        ["linear"],
        ["coalesce", ["get", "vc_ratio"], 0],
        0, "#3b82f6",
        0.3, "#22c55e",
        0.5, "#eab308",
        0.7, "#f97316",
        1.0, "#ef4444",
        1.5, "#991b1b",
      ],
      "line-width": [
        "match",
        ["get", "road_type"],
        "motorway", 3,
        "trunk", 2.5,
        "primary", 2,
        "secondary", 1.8,
        "tertiary", 1.5,
        1,
      ],
      "line-opacity": 0.6,
    },
  };

  const routeLayer: LayerSpecification = {
    id: "route-options",
    type: "line",
    source: "route-options",
    paint: {
      "line-color": [
        "match",
        ["get", "routeType"],
        "Selfish", "#3b82f6",
        "SystemSuggested", "#22c55e",
        "Alternative", "#a855f7",
        "#ffffff",
      ],
      "line-width": [
        "case",
        ["==", ["get", "active"], 1], 6,
        3,
      ],
      "line-opacity": [
        "case",
        ["==", ["get", "active"], 1], 1,
        0.6,
      ],
    },
  };

  const aiCarsLayer: LayerSpecification = {
    id: "ai-cars",
    type: "circle",
    source: "ai-cars",
    paint: {
      "circle-radius": 3.5,
      "circle-color": "#60a5fa",
      "circle-opacity": 0.8,
      "circle-stroke-width": 0.5,
      "circle-stroke-color": "#1e3a5f",
    },
  };

  const playerCarLayer: LayerSpecification = {
    id: "player-car",
    type: "circle",
    source: "player-car",
    paint: {
      "circle-radius": 7,
      "circle-color": "#f59e0b",
      "circle-stroke-width": 2,
      "circle-stroke-color": "#ffffff",
    },
  };

  return (
    <Map
      ref={mapRef}
      initialViewState={{
        longitude: city.center[1],
        latitude: city.center[0],
        zoom: city.zoom,
      }}
      style={{ width: "100%", height: "100vh", cursor: clickable ? "crosshair" : "grab" }}
      mapStyle={MAP_STYLE}
      onClick={handleClick}
    >
      <NavigationControl position="bottom-right" />

      {/* Road network */}
      <Source id="road-network" type="geojson" data={geojson ?? EMPTY_FC}>
        <Layer {...networkLayer} />
      </Source>

      {/* Route options */}
      <Source id="route-options" type="geojson" data={routeGeoJSON}>
        <Layer {...routeLayer} />
      </Source>

      {/* AI cars */}
      <Source id="ai-cars" type="geojson" data={EMPTY_FC}>
        <Layer {...aiCarsLayer} />
      </Source>

      {/* Player car */}
      <Source id="player-car" type="geojson" data={EMPTY_FC}>
        <Layer {...playerCarLayer} />
      </Source>
    </Map>
  );
}
