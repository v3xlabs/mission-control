import { useQueryClient } from "@tanstack/react-query";
import { useStatus } from "./useStatus";
import type { components } from "../api/schema.gen";

type DeviceStatus = components["schemas"]["DeviceStatus"];

export const useCurrentPlaylist = () => {
  const {data: status, ...rest} = useStatus();

  return {
    data: status?.current_playlist,
    ...rest
  };
}; 