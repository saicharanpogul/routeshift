"use client";

import { CityConfig } from "@/types/graph";
import { CITIES } from "@/lib/cities";

interface CitySelectorProps {
  selected: CityConfig;
  onSelect: (city: CityConfig) => void;
}

export function CitySelector({ selected, onSelect }: CitySelectorProps) {
  return (
    <div className="flex gap-1">
      {CITIES.map((city) => (
        <button
          key={city.name}
          onClick={() => onSelect(city)}
          className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
            selected.name === city.name
              ? "bg-white text-black font-medium"
              : "bg-white/10 text-white hover:bg-white/20"
          }`}
        >
          {city.label}
        </button>
      ))}
    </div>
  );
}
