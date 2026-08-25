import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { baseUrl } from "../api/api";
import type { components } from "../api/schema.gen";

type DeviceStatus = components["schemas"]["DeviceStatus"];

type DisplayEvent = {
  current_playlist_id: string | null;
  current_tab_id: string | null;
  auto_rotate: boolean;
  screen_on: boolean;
};

export const useDisplayEvents = () => {
  const client = useQueryClient();

  useEffect(() => {
    const source = new EventSource(new URL("events", baseUrl));

    source.addEventListener("message", (event) => {
      const update = JSON.parse(event.data) as DisplayEvent;

      client.setQueryData(["status"], (status: DeviceStatus | undefined) => status && {
        ...status,
        ...update,
      });
    });

    return () => source.close();
  }, [client]);
};
