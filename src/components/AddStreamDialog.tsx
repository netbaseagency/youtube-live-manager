import { useState } from "react";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Select } from "./ui/select";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "./ui/dialog";
import type { Stream, ScheduleType, ScheduleConfig } from "../types";
import { FolderOpen, Key, Video, Clock, Globe, Zap, Timer } from "lucide-react";

interface AddStreamDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (stream: Omit<Stream, "id" | "status" | "startedAt" | "stoppedAt" | "elapsedSeconds" | "lastElapsedSeconds"> & { startImmediately: boolean }) => void;
}

const TIMEZONES = [
  "Asia/Ho_Chi_Minh",
  "Asia/Bangkok",
  "Asia/Singapore",
  "Asia/Tokyo",
  "America/New_York",
  "America/Los_Angeles",
  "Europe/London",
  "Europe/Paris",
  "UTC",
];

export function AddStreamDialog({ open, onOpenChange, onSubmit }: AddStreamDialogProps) {
  const [name, setName] = useState("");
  const [youtubeKey, setYoutubeKey] = useState("");
  const [videoPath, setVideoPath] = useState("");
  const [startImmediately, setStartImmediately] = useState(true);
  const [scheduleType, setScheduleType] = useState<ScheduleType>("manual");
  
  const [hours, setHours] = useState(0);
  const [minutes, setMinutes] = useState(0);
  const [seconds, setSeconds] = useState(0);
  
  const [stopDate, setStopDate] = useState("");
  const [stopTime, setStopTime] = useState("");
  const [timezone, setTimezone] = useState("Asia/Ho_Chi_Minh");

  const handleSelectFile = async () => {
    try {
      const { open: openDialog } = await import("@tauri-apps/plugin-dialog");
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: "Video", extensions: ["mp4", "mkv", "avi", "mov", "webm"] }],
      });
      if (selected) {
        setVideoPath(selected as string);
      }
    } catch (error) {
      console.error("Không thể chọn file:", error);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    let schedule: ScheduleConfig = { type: scheduleType };
    
    if (scheduleType === "duration") {
      schedule.duration = { hours, minutes, seconds };
    } else if (scheduleType === "absolute") {
      // Combine date and time into ISO datetime
      const datetime = stopDate && stopTime ? `${stopDate}T${stopTime}` : "";
      schedule.absolute = { datetime, timezone };
    }

    onSubmit({
      name,
      youtubeKey,
      videoPath,
      schedule,
      createdAt: new Date().toISOString(),
      startImmediately,
    });

    // Reset form
    setName("");
    setYoutubeKey("");
    setVideoPath("");
    setStartImmediately(true);
    setScheduleType("manual");
    setHours(0);
    setMinutes(0);
    setSeconds(0);
    setStopDate("");
    setStopTime("");
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md" onClose={() => onOpenChange(false)}>
        <DialogHeader>
          <DialogTitle>Thêm Luồng Mới</DialogTitle>
          <DialogDescription>
            Cấu hình thông tin luồng phát live YouTube.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-3">
          {/* Stream Name */}
          <div className="space-y-1">
            <Label htmlFor="name">Tên luồng</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Luồng Live của tôi"
              required
            />
          </div>

          {/* YouTube Key */}
          <div className="space-y-1">
            <Label htmlFor="youtubeKey">
              <Key className="h-3 w-3 text-slate-400" />
              Stream Key YouTube
            </Label>
            <Input
              id="youtubeKey"
              value={youtubeKey}
              onChange={(e) => setYoutubeKey(e.target.value)}
              placeholder="xxxx-xxxx-xxxx-xxxx-xxxx"
              required
              className="font-mono"
            />
          </div>

          {/* Video File */}
          <div className="space-y-1">
            <Label htmlFor="videoPath">
              <Video className="h-3 w-3 text-slate-400" />
              File Video
            </Label>
            <div className="flex gap-1.5">
              <Input
                id="videoPath"
                value={videoPath}
                onChange={(e) => setVideoPath(e.target.value)}
                placeholder="/đường/dẫn/video.mp4"
                required
                className="flex-1"
              />
              <Button type="button" variant="outline" size="icon-sm" onClick={handleSelectFile}>
                <FolderOpen className="h-3.5 w-3.5" />
              </Button>
            </div>
          </div>

          {/* Start Time */}
          <div className="space-y-2">
            <Label>Thời gian bắt đầu</Label>
            <div className="grid grid-cols-2 gap-2">
              <button
                type="button"
                onClick={() => setStartImmediately(true)}
                className={`flex items-center justify-center gap-2 p-2.5 rounded-lg border transition-all ${
                  startImmediately 
                    ? "border-red-500 bg-red-50 text-red-700" 
                    : "border-slate-200 bg-white text-slate-600 hover:border-slate-300"
                }`}
              >
                <Zap className="h-4 w-4" />
                <span className="font-medium">Ngay lập tức</span>
              </button>
              <button
                type="button"
                onClick={() => setStartImmediately(false)}
                className={`flex items-center justify-center gap-2 p-2.5 rounded-lg border transition-all ${
                  !startImmediately 
                    ? "border-red-500 bg-red-50 text-red-700" 
                    : "border-slate-200 bg-white text-slate-600 hover:border-slate-300"
                }`}
              >
                <Timer className="h-4 w-4" />
                <span className="font-medium">Phát sau</span>
              </button>
            </div>
            <p className="text-[10px] text-slate-500">
              {startImmediately 
                ? "Luồng sẽ tự động phát ngay sau khi lưu" 
                : "Luồng sẽ được lưu ở trạng thái nháp"}
            </p>
          </div>

          {/* Schedule Type */}
          <div className="space-y-1">
            <Label htmlFor="scheduleType">
              <Clock className="h-3 w-3 text-slate-400" />
              Hẹn giờ dừng
            </Label>
            <Select
              id="scheduleType"
              value={scheduleType}
              onChange={(e) => setScheduleType(e.target.value as ScheduleType)}
            >
              <option value="manual">Dừng thủ công</option>
              <option value="duration">Sau thời lượng</option>
              <option value="absolute">Vào giờ cụ thể</option>
            </Select>
          </div>

          {/* Duration Fields */}
          {scheduleType === "duration" && (
            <div className="rounded-lg bg-slate-50 p-3 border border-slate-200 space-y-2">
              <Label className="text-slate-500">Dừng sau</Label>
              <div className="grid grid-cols-3 gap-2">
                <div className="space-y-1">
                  <Input
                    type="number"
                    min={0}
                    max={99}
                    value={hours}
                    onChange={(e) => setHours(parseInt(e.target.value) || 0)}
                    className="text-center font-mono"
                  />
                  <p className="text-[10px] text-slate-400 text-center">Giờ</p>
                </div>
                <div className="space-y-1">
                  <Input
                    type="number"
                    min={0}
                    max={59}
                    value={minutes}
                    onChange={(e) => setMinutes(parseInt(e.target.value) || 0)}
                    className="text-center font-mono"
                  />
                  <p className="text-[10px] text-slate-400 text-center">Phút</p>
                </div>
                <div className="space-y-1">
                  <Input
                    type="number"
                    min={0}
                    max={59}
                    value={seconds}
                    onChange={(e) => setSeconds(parseInt(e.target.value) || 0)}
                    className="text-center font-mono"
                  />
                  <p className="text-[10px] text-slate-400 text-center">Giây</p>
                </div>
              </div>
            </div>
          )}

          {/* Absolute Time Fields */}
          {scheduleType === "absolute" && (
            <div className="rounded-lg bg-slate-50 p-3 border border-slate-200 space-y-3">
              <Label className="text-slate-500">Dừng vào lúc</Label>
              <div className="grid grid-cols-2 gap-2">
                <div className="space-y-1">
                  <Label htmlFor="stopDate" className="text-[10px] text-slate-400">Ngày</Label>
                  <Input
                    id="stopDate"
                    type="date"
                    value={stopDate}
                    onChange={(e) => setStopDate(e.target.value)}
                    required={scheduleType === "absolute"}
                    className="font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label htmlFor="stopTime" className="text-[10px] text-slate-400">Giờ</Label>
                  <Input
                    id="stopTime"
                    type="time"
                    value={stopTime}
                    onChange={(e) => setStopTime(e.target.value)}
                    required={scheduleType === "absolute"}
                    className="font-mono"
                  />
                </div>
              </div>
              <div className="space-y-1">
                <Label htmlFor="timezone" className="text-[10px] text-slate-400">
                  <Globe className="h-2.5 w-2.5" />
                  Múi giờ
                </Label>
                <Select
                  id="timezone"
                  value={timezone}
                  onChange={(e) => setTimezone(e.target.value)}
                >
                  {TIMEZONES.map((tz) => (
                    <option key={tz} value={tz}>
                      {tz.replace(/_/g, " ")}
                    </option>
                  ))}
                </Select>
              </div>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-2 pt-2">
            <Button 
              type="button" 
              variant="outline" 
              size="sm"
              className="flex-1" 
              onClick={() => onOpenChange(false)}
            >
              Hủy
            </Button>
            <Button type="submit" size="sm" className="flex-1">
              {startImmediately ? "Lưu & Phát" : "Lưu nháp"}
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}
