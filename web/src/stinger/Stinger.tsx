import { type FC, useEffect, useState } from "react";

type StingerInfo = { name: string; file: string; };

/**
 * Plays one clip, full bleed, on black.
 *
 * The daemon decides how long this page stays up; the clip only has to fill that time. Nothing
 * here waits on the API before painting, so the screen goes black immediately rather than showing
 * whatever was there while the file is fetched.
 */
export const Stinger: FC<{ name: string; }> = ({ name }) => {
  const [hasFailed, setHasFailed] = useState(false);
  const [file, setFile] = useState<string | undefined>();

  useEffect(() => {
    let isCancelled = false;

    const resolve = async () => {
      try {
        const response = await fetch("/api/stingers");
        const stingers = await response.json() as StingerInfo[];

        if (!isCancelled) {
          setFile(stingers.find(stinger => stinger.name === name)?.file);
        }
      }
      catch {
        setHasFailed(true);
      }
    };

    resolve();

    return () => {
      isCancelled = true;
    };
  }, [name]);

  // Black rather than a message. A failed transition should look like a cut, not like an error
  // on the wall.
  return (
    <main className="h-screen w-screen bg-black">
      {file && !hasFailed && (
        <video
          src={`/api/media/${encodeURIComponent(file)}`}
          autoPlay
          muted
          playsInline
          onError={() => setHasFailed(true)}
          className="h-full w-full object-cover"
        />
      )}
    </main>
  );
};
