import { FC } from "react";
import { PlaylistList } from "../sections/PlaylistList";

const App: FC<{}> = ({}) => {
  return (
    <div className="min-h-screen p-4 bg-gray-950 text-gray-100">
      <PlaylistList />
    </div>
  );
};

export default App;
