import { type FC } from "react";

import { StatusBar } from "../components/StatusBar";
import { useDisplayEvents } from "../hooks/useDisplayEvents";
import { PlaylistList } from "../sections/PlaylistList";

export const App: FC = () => {
  useDisplayEvents();

  return (
    <div className="min-h-screen bg-gray-950 text-gray-100">
      <StatusBar />
      <PlaylistList />
    </div>
  );
};
