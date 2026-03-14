"use client";

import { useRef, useMemo } from "react";
import {
  Map,
  Source,
  Layer,
  NavigationControl,
  type MapRef,
} from "@vis.gl/react-maplibre";
import type { LayerSpecification } from "maplibre-gl";
import { CityConfig } from "@/types/graph";
import { AssignmentResult } from "@/types/graph";

const MAP_STYLE = "https://tiles.openfreemap.org/styles/liberty";

interface MapViewProps {
  city: CityConfig;
  geojson: GeoJSON.FeatureCollection | null;
  result: AssignmentResult | null;
}

export function MapView({ city, geojson, result }: MapViewProps) {
  const mapRef = useRef<MapRef>(null);

  const styledGeojson = useMemo(() => {
    if (!geojson || !result) return geojson;

    const features = geojson.features.map((feature, i) => {
      const edgeId = feature.properties?.edge_id ?? i;
      const flow = result.edge_flows[edgeId] ?? 0;
      const travelTime = result.edge_travel_times[edgeId] ?? 0;
      const freeFlowTime =
        (feature.properties?.free_flow_time as number) ?? travelTime;
      const capacity = (feature.properties?.capacity as number) ?? 1000;

      const vcRatio = capacity > 0 ? flow / capacity : 0;

      return {
        ...feature,
        properties: {
          ...feature.properties,
          flow,
          travel_time: travelTime,
          free_flow_time: freeFlowTime,
          vc_ratio: vcRatio,
        },
      };
    });

    return { ...geojson, features };
  }, [geojson, result]);

  const networkLayer: LayerSpecification = {
    id: "road-network",
    type: "line",
    source: "road-network",
    paint: {
      "line-color": result
        ? [
            "interpolate",
            ["linear"],
            ["get", "vc_ratio"],
            0,
            "#22c55e", // green
            0.3,
            "#22c55e",
            0.5,
            "#eab308", // yellow
            0.7,
            "#f97316", // orange
            1.0,
            "#ef4444", // red
            1.5,
            "#991b1b", // dark red
          ]
        : "#3b82f6", // blue when no result
      "line-width": [
        "match",
        ["get", "road_type"],
        "motorway",
        4,
        "trunk",
        3.5,
        "primary",
        3,
        "secondary",
        2.5,
        "tertiary",
        2,
        1.5,
      ],
      "line-opacity": 0.8,
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
      style={{ width: "100%", height: "100vh" }}
      mapStyle={MAP_STYLE}
    >
      <NavigationControl position="bottom-right" />

      {styledGeojson && (
        <Source id="road-network" type="geojson" data={styledGeojson}>
          <Layer {...networkLayer} />
        </Source>
      )}
    </Map>
  );
}
