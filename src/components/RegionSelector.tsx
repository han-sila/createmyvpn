import { AWS_REGIONS } from "../lib/types";

interface Props {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

function RegionSelector({ value, onChange, disabled }: Props) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      disabled={disabled}
      style={{ colorScheme: "dark" }}
      className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent disabled:opacity-50"
    >
      {AWS_REGIONS.map((region) => (
        <option
          key={region.code}
          value={region.code}
          className="bg-gray-800 text-white"
        >
          {region.name} ({region.code})
        </option>
      ))}
    </select>
  );
}

export default RegionSelector;
