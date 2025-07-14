import { useQuery } from "@tanstack/react-query";

interface DeviceStatus {
  device_id: string;
  device_name: string;
  current_playlist?: string | null;
  current_tab?: string | null;
  uptime_seconds: number;
}

const fetchStatus = async (): Promise<DeviceStatus> => {
  const res = await fetch("/api/status");
  if (!res.ok) throw new Error("Failed to fetch status");
  return res.json();
};

export const useStatus = () => {
  return useQuery({
    queryKey: ["status"],
    queryFn: fetchStatus,
    refetchInterval: 2000,
  });
}; 