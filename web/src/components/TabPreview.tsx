import { type FC, useState } from "react";

import { baseUrl } from "../api/api";

/**
 * A tab has no frame until its page has painted at least once, so a tab the rotation has not
 * reached yet has nothing to show. Saying so beats an empty box the reader reads as broken.
 */
export const TabPreview: FC<{ tabId: string; }> = ({ tabId }) => {
  const [isAvailable, setIsAvailable] = useState(true);

  // Built from the URL this page was served from, so the stream follows the host the reader
  // actually typed rather than whatever the daemon believes it is called.
  const source = new URL(`preview_live/${encodeURIComponent(tabId)}`, baseUrl);

  return (
    <>
      {/* A multipart stream never finishes loading, so `load` is no signal that it works.
                Only the failure is observable, and that is what the message reports. */}
      <img
        src={source.href}
        alt=""
        onError={() => setIsAvailable(false)}
        onLoad={() => setIsAvailable(true)}
        className="h-full w-full object-cover"
      />
      {!isAvailable && (
        <span className="absolute inset-0 flex items-center justify-center px-3 text-center text-xs text-gray-500">
          not rendered yet
        </span>
      )}
    </>
  );
};
