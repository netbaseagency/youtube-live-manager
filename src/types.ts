export type StreamStatus = "idle" | "live" | "scheduled" | "completed" | "error" | "stopping";

export type ScheduleType = "manual" | "duration" | "absolute";

export interface ScheduleConfig {
  type: ScheduleType;
  duration?: {
    hours: number;
    minutes: number;
    seconds: number;
  };
  absolute?: {
    datetime: string;
    timezone: string;
  };
}

export interface Stream {
  id: string;
  name: string;
  youtubeKey: string;
  videoPath: string;
  status: StreamStatus;
  schedule: ScheduleConfig;
  startedAt?: string;
  stoppedAt?: string;
  createdAt: string;
  elapsedSeconds?: number;
  lastElapsedSeconds?: number;
}

export interface StreamInput {
  name: string;
  youtubeKey: string;
  videoPath: string;
  schedule: ScheduleConfig;
  createdAt: string;
  startImmediately: boolean;
}
