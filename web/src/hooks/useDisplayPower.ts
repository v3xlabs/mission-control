import { useMutation, useQueryClient } from "@tanstack/react-query";

import { apiRequest } from "../api/api";
import { assertRequest } from "../api/request";

export const useDisplayPower = () => {
  const client = useQueryClient();

  return useMutation({
    mutationFn: async (isOn: boolean) => assertRequest(await apiRequest("/display/power/{on}", "post", {
      path: { on: isOn },
    })),
    onSuccess: () => {
      client.invalidateQueries({ queryKey: ["status"] });
    },
  });
};
