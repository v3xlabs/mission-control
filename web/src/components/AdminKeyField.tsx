import { useQueryClient } from "@tanstack/react-query";
import { type FC, useState } from "react";
import { FiLock, FiUnlock } from "react-icons/fi";

import { adminKey, setAdminKey } from "../api/auth";

type Properties = {
  requiresAuth: boolean;
  authenticated: boolean;
};

export const AdminKeyField: FC<Properties> = ({ requiresAuth, authenticated }) => {
  const client = useQueryClient();
  const [key, setKey] = useState(adminKey());

  if (!requiresAuth) {
    return (
      <span
        className="flex items-center gap-1.5 bg-amber-700 px-2 py-1 text-xs text-white"
        title="No admin key is configured on the daemon, so anything that can reach this port can change the display."
      >
        <FiUnlock />
        unauthenticated
      </span>
    );
  }

  return (
    <label
      className="flex items-center gap-1.5"
      title={authenticated ? "Key accepted" : "Mutations will be refused until this key matches"}
    >
      <FiLock className={authenticated ? "text-emerald-400" : "text-red-400"} />
      <input
        type="password"
        value={key}
        onChange={(event) => {
          setKey(event.target.value);
          setAdminKey(event.target.value);
          // The daemon reports whether the key it just saw was accepted, so refetching
          // status is what turns the indicator green.
          client.invalidateQueries({ queryKey: ["status"] });
        }}
        placeholder="admin key"
        className={[
          "w-40 border bg-gray-900 px-2 py-1 text-sm text-gray-100",
          authenticated ? "border-gray-800" : "border-red-800",
        ].join(" ")}
      />
    </label>
  );
};
