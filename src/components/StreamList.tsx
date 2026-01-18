import { useState } from "react";
import { Play, Square, Trash2, Pencil, FileVideo, Radio, Check, Minus, ChevronLeft, ChevronRight, CheckCircle } from "lucide-react";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import type { Stream } from "../types";

interface StreamListProps {
  streams: Stream[];
  onStart: (id: string) => void;
  onStop: (id: string) => void;
  onDelete: (id: string) => void;
  onEdit?: (stream: Stream) => void;
}

const PAGE_SIZE = 20;

function formatTime(dateStr?: string): string {
  if (!dateStr) return "-";
  const date = new Date(dateStr);
  return date.toLocaleTimeString("vi-VN", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

function formatDate(dateStr?: string): string {
  if (!dateStr) return "-";
  const date = new Date(dateStr);
  return date.toLocaleDateString("vi-VN", { day: "2-digit", month: "2-digit" });
}

function formatElapsed(seconds?: number): string {
  if (!seconds) return "-";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  return `${h.toString().padStart(2, "0")}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
}

function getStatusBadge(status: Stream["status"]) {
  switch (status) {
    case "live":
      return (
        <Badge variant="live" className="gap-1">
          <Radio className="h-2.5 w-2.5 animate-pulse-live fill-current" />
          Đang phát
        </Badge>
      );
    case "completed":
      return (
        <Badge variant="success" className="gap-1">
          <CheckCircle className="h-2.5 w-2.5" />
          Đã phát
        </Badge>
      );
    case "scheduled":
      return <Badge variant="warning">Đã hẹn</Badge>;
    case "error":
      return <Badge variant="destructive">Lỗi</Badge>;
    case "stopping":
      return <Badge variant="secondary">Đang dừng</Badge>;
    default:
      return <Badge variant="outline">Nháp</Badge>;
  }
}

export function StreamList({ 
  streams, 
  onStart, 
  onStop, 
  onDelete, 
  onEdit
}: StreamListProps) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [currentPage, setCurrentPage] = useState(1);

  const totalPages = Math.ceil(streams.length / PAGE_SIZE);
  const startIndex = (currentPage - 1) * PAGE_SIZE;
  const paginatedStreams = streams.slice(startIndex, startIndex + PAGE_SIZE);

  const toggleSelect = (id: string) => {
    const newSelected = new Set(selectedIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedIds(newSelected);
  };

  const toggleSelectAll = () => {
    if (selectedIds.size === paginatedStreams.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(paginatedStreams.map(s => s.id)));
    }
  };

  const handleBatchStart = () => {
    Array.from(selectedIds).forEach(id => {
      const stream = streams.find(s => s.id === id);
      if (stream && (stream.status === "idle" || stream.status === "error")) {
        onStart(id);
      }
    });
    setSelectedIds(new Set());
  };

  const handleBatchStop = () => {
    Array.from(selectedIds).forEach(id => {
      const stream = streams.find(s => s.id === id);
      if (stream && stream.status === "live") {
        onStop(id);
      }
    });
    setSelectedIds(new Set());
  };

  const handleBatchDelete = () => {
    Array.from(selectedIds).forEach(id => {
      const stream = streams.find(s => s.id === id);
      if (stream && stream.status !== "live" && stream.status !== "stopping") {
        onDelete(id);
      }
    });
    setSelectedIds(new Set());
  };

  const isAllSelected = paginatedStreams.length > 0 && selectedIds.size === paginatedStreams.length;
  const isSomeSelected = selectedIds.size > 0 && selectedIds.size < paginatedStreams.length;

  if (streams.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <div className="mb-4 rounded-xl bg-slate-100 p-4">
          <FileVideo className="h-8 w-8 text-slate-400 fill-slate-200" />
        </div>
        <h3 className="mb-1 font-semibold text-slate-900">Chưa có luồng nào</h3>
        <p className="text-slate-500 max-w-xs">
          Nhấn "Thêm Luồng" để tạo luồng phát live YouTube đầu tiên.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {/* Batch Actions */}
      {selectedIds.size > 0 && (
        <div className="flex items-center gap-2 p-2 bg-slate-50 rounded-lg border border-slate-200">
          <span className="text-slate-600 mr-2">
            Đã chọn {selectedIds.size} luồng
          </span>
          <Button variant="success" size="sm" onClick={handleBatchStart}>
            <Play className="mr-1 h-3 w-3 fill-current" />
            Bắt đầu
          </Button>
          <Button variant="destructive" size="sm" onClick={handleBatchStop}>
            <Square className="mr-1 h-3 w-3 fill-current" />
            Dừng
          </Button>
          <Button variant="outline" size="sm" onClick={handleBatchDelete}>
            <Trash2 className="mr-1 h-3 w-3" />
            Xóa
          </Button>
        </div>
      )}

      {/* Table */}
      <div className="rounded-lg border border-slate-200 overflow-x-auto bg-white">
        <table className="w-full">
          <thead className="bg-slate-50 border-b border-slate-200">
            <tr>
              <th className="w-10 px-3 py-2 text-center">
                <button
                  onClick={toggleSelectAll}
                  className="flex items-center justify-center w-4 h-4 rounded border border-slate-300 bg-white hover:border-red-400 transition-colors mx-auto"
                >
                  {isAllSelected && <Check className="h-3 w-3 text-red-500" />}
                  {isSomeSelected && <Minus className="h-3 w-3 text-red-500" />}
                </button>
              </th>
              <th className="w-12 px-2 py-2 text-left font-medium text-slate-600">STT</th>
              <th className="px-3 py-2 text-left font-medium text-slate-600">Tên luồng</th>
              <th className="w-24 px-3 py-2 text-left font-medium text-slate-600">Bắt đầu</th>
              <th className="w-24 px-3 py-2 text-left font-medium text-slate-600">Kết thúc</th>
              <th className="w-24 px-3 py-2 text-center font-medium text-slate-600">Thời lượng</th>
              <th className="w-24 px-3 py-2 text-left font-medium text-slate-600">Trạng thái</th>
              <th className="w-28 px-3 py-2 text-center font-medium text-slate-600">Thao tác</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {paginatedStreams.map((stream, index) => (
              <tr 
                key={stream.id} 
                className={`hover:bg-slate-50 transition-colors ${
                  stream.status === "live" ? "bg-red-50/50" : ""
                } ${selectedIds.has(stream.id) ? "bg-red-50" : ""}`}
              >
                {/* Checkbox */}
                <td className="px-3 py-2 text-center">
                  <button
                    onClick={() => toggleSelect(stream.id)}
                    className={`flex items-center justify-center w-4 h-4 rounded border transition-colors mx-auto ${
                      selectedIds.has(stream.id) 
                        ? "border-red-500 bg-red-500" 
                        : "border-slate-300 bg-white hover:border-red-400"
                    }`}
                  >
                    {selectedIds.has(stream.id) && <Check className="h-3 w-3 text-white" />}
                  </button>
                </td>

                {/* STT */}
                <td className="px-2 py-2 text-slate-400">{startIndex + index + 1}</td>

                {/* Name + File */}
                <td className="px-3 py-2">
                  <div className="font-medium text-slate-900" title={stream.name}>
                    {stream.name}
                  </div>
                  <div className="text-[10px] text-slate-400 truncate max-w-[280px]" title={stream.videoPath}>
                    {stream.videoPath.split("/").pop()}
                  </div>
                </td>

                {/* Start Time */}
                <td className="px-3 py-2 text-slate-600">
                  {stream.startedAt ? (
                    <div>
                      <div>{formatTime(stream.startedAt)}</div>
                      <div className="text-[10px] text-slate-400">{formatDate(stream.startedAt)}</div>
                    </div>
                  ) : (
                    <span className="text-slate-300">-</span>
                  )}
                </td>

                {/* End Time - Only show when stream has stopped */}
                <td className="px-3 py-2 text-slate-600">
                  {stream.stoppedAt && stream.status !== "live" ? (
                    <div>
                      <div>{formatTime(stream.stoppedAt)}</div>
                      <div className="text-[10px] text-slate-400">{formatDate(stream.stoppedAt)}</div>
                    </div>
                  ) : (
                    <span className="text-slate-300">-</span>
                  )}
                </td>

                {/* Duration */}
                <td className="px-3 py-2 text-center">
                  {stream.elapsedSeconds !== undefined && stream.elapsedSeconds > 0 ? (
                    <span className={`font-mono ${stream.status === "live" ? "text-red-600 font-medium" : "text-slate-600"}`}>
                      {formatElapsed(stream.elapsedSeconds)}
                    </span>
                  ) : (
                    <span className="text-slate-300">-</span>
                  )}
                </td>

                {/* Status */}
                <td className="px-3 py-2">
                  {getStatusBadge(stream.status)}
                </td>

                {/* Actions */}
                <td className="px-3 py-2">
                  <div className="flex items-center justify-center gap-1">
                    {stream.status === "idle" || stream.status === "error" || stream.status === "completed" ? (
                      <Button
                        variant="success"
                        size="icon-sm"
                        onClick={() => onStart(stream.id)}
                        title="Bắt đầu"
                      >
                        <Play className="h-4 w-4 fill-current" />
                      </Button>
                    ) : stream.status === "live" ? (
                      <Button
                        variant="destructive"
                        size="icon-sm"
                        onClick={() => onStop(stream.id)}
                        title="Dừng"
                      >
                        <Square className="h-4 w-4 fill-current" />
                      </Button>
                    ) : (
                      <Button variant="secondary" size="icon-sm" disabled>
                        <Square className="h-4 w-4 fill-current" />
                      </Button>
                    )}
                    
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => onEdit?.(stream)}
                      disabled={stream.status === "live"}
                      title="Sửa"
                      className="text-slate-400 hover:text-blue-500"
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>

                    <Button
                      variant="ghost"
                      size="icon-sm"
                      onClick={() => onDelete(stream.id)}
                      disabled={stream.status === "live" || stream.status === "stopping"}
                      title="Xóa"
                      className="text-slate-400 hover:text-red-500"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between px-2">
          <div className="text-slate-500">
            Hiển thị {startIndex + 1}-{Math.min(startIndex + PAGE_SIZE, streams.length)} / {streams.length} luồng
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant="outline"
              size="icon-sm"
              onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
              disabled={currentPage === 1}
            >
              <ChevronLeft className="h-3 w-3" />
            </Button>
            <span className="text-slate-600 px-2">
              {currentPage} / {totalPages}
            </span>
            <Button
              variant="outline"
              size="icon-sm"
              onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
              disabled={currentPage === totalPages}
            >
              <ChevronRight className="h-3 w-3" />
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
