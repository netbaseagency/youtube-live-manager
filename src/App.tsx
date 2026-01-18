import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { StreamList } from "./components/StreamList";
import { AddStreamDialog } from "./components/AddStreamDialog";
import { Header } from "./components/Header";
import type { Stream } from "./types";

function App() {
  const [streams, setStreams] = useState<Stream[]>([]);
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [instanceId, setInstanceId] = useState<string>("");

  useEffect(() => {
    // Get or create instance ID for multi-window support
    let id = localStorage.getItem("instance_id");
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem("instance_id", id);
    }
    setInstanceId(id);
    
    // Initialize and load streams
    initializeApp(id);
  }, []);

  const initializeApp = async (id: string) => {
    try {
      await invoke("initialize", { instanceId: id });
      await loadStreams();
    } catch (error) {
      console.error("Failed to initialize:", error);
    }
  };

  const loadStreams = async () => {
    try {
      const result = await invoke<Stream[]>("get_streams");
      setStreams(result);
    } catch (error) {
      console.error("Failed to load streams:", error);
    }
  };

  const handleAddStream = async (stream: Omit<Stream, "id" | "status" | "startedAt" | "stoppedAt" | "elapsedSeconds" | "lastElapsedSeconds"> & { startImmediately: boolean }) => {
    try {
      await invoke("add_stream", { stream });
      await loadStreams();
      setIsAddDialogOpen(false);
    } catch (error) {
      console.error("Failed to add stream:", error);
    }
  };

  const handleStartStream = async (id: string) => {
    try {
      await invoke("start_stream", { id });
      await loadStreams();
    } catch (error) {
      console.error("Failed to start stream:", error);
    }
  };

  const handleStopStream = async (id: string) => {
    try {
      await invoke("stop_stream", { id });
      await loadStreams();
    } catch (error) {
      console.error("Failed to stop stream:", error);
    }
  };

  const handleDeleteStream = async (id: string) => {
    try {
      await invoke("delete_stream", { id });
      await loadStreams();
    } catch (error) {
      console.error("Failed to delete stream:", error);
    }
  };

  // Poll stream status every 2 seconds
  useEffect(() => {
    const interval = setInterval(loadStreams, 2000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-slate-50">
      <Header 
        instanceId={instanceId} 
        onAddClick={() => setIsAddDialogOpen(true)} 
        streamCount={streams.length}
        liveCount={streams.filter(s => s.status === "live").length}
      />
      
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
        <StreamList
          streams={streams}
          onStart={handleStartStream}
          onStop={handleStopStream}
          onDelete={handleDeleteStream}
        />
      </main>

      <AddStreamDialog
        open={isAddDialogOpen}
        onOpenChange={setIsAddDialogOpen}
        onSubmit={handleAddStream}
      />
    </div>
  );
}

export default App;
