import { FC, useEffect, useState } from "react";

interface ProgressCircleProps {
  /** Total duration in seconds */
  duration: number;
  /** When the timer started (seconds since epoch) */
  startTime: number;
  /** Size of the circle in pixels */
  size?: number;
  /** Stroke width */
  strokeWidth?: number;
  /** Show percentage text in center */
  showText?: boolean;
  /** Color of the progress stroke */
  color?: string;
  /** Callback to trigger when progress completes */
  onComplete?: () => void;
}

export const ProgressCircle: FC<ProgressCircleProps> = ({
  duration,
  startTime,
  size = 24,
  strokeWidth = 2,
  showText = false,
  color = "#10b981", // green-500
  onComplete
}) => {
  const [progress, setProgress] = useState(0);
  const [timeLeft, setTimeLeft] = useState(duration);
  const [completionTriggered, setCompletionTriggered] = useState(false);

  useEffect(() => {
    const updateProgress = () => {
      const now = Math.floor(Date.now() / 1000);
      const elapsed = now - startTime;
      const newProgress = Math.min(elapsed / duration, 1);
      const newTimeLeft = Math.max(duration - elapsed, 0);
      
      setProgress(newProgress);
      setTimeLeft(newTimeLeft);

      // Trigger onComplete 10ms after completion
      if (newProgress >= 1 && !completionTriggered && onComplete) {
        setCompletionTriggered(true);
        setTimeout(() => {
          onComplete();
        }, 10);
      }
    };

    updateProgress(); // Initial update
    const interval = setInterval(updateProgress, 1000);

    return () => clearInterval(interval);
  }, [duration, startTime, completionTriggered, onComplete]);

  const radius = (size - strokeWidth) / 2;
  const circumference = radius * 2 * Math.PI;
  const strokeDasharray = circumference;
  const strokeDashoffset = circumference - (progress * circumference);

  return (
    <div className="relative inline-flex items-center justify-center">
      <svg
        width={size}
        height={size}
        className="transform -rotate-90"
      >
        {/* Background circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="transparent"
          stroke="currentColor"
          strokeWidth={strokeWidth}
          className="text-gray-600"
        />
        {/* Progress circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="transparent"
          stroke={color}
          strokeWidth={strokeWidth}
          strokeDasharray={strokeDasharray}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          className="transition-all duration-1000 ease-out"
        />
      </svg>
      {showText && (
        <div className="absolute inset-0 flex items-center justify-center">
          <span className="text-xs font-medium text-gray-300">
            {Math.ceil(timeLeft)}
          </span>
        </div>
      )}
    </div>
  );
};