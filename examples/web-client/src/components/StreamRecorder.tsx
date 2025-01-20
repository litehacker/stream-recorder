import React, { useEffect, useRef, useState } from "react";
import axios from "axios";

interface StreamRecorderProps {
  apiKey: string;
  onError: (error: Error) => void;
}

interface RoomConfig {
  video_codec: string;
  audio_codec: string;
  max_bitrate: number;
  frame_rate: number;
  resolution: string;
  deduplication_enabled: boolean;
  adaptive_bitrate: boolean;
  enable_frame_batching: boolean;
  enable_hardware_acceleration: boolean;
}

export const StreamRecorder: React.FC<StreamRecorderProps> = ({
  apiKey,
  onError,
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [roomId, setRoomId] = useState<string | null>(null);
  const [stats, setStats] = useState<{
    fps: number;
    bitrate: number;
    frameCount: number;
    deduplicationRatio: number;
  }>({
    fps: 0,
    bitrate: 0,
    frameCount: 0,
    deduplicationRatio: 0,
  });

  const videoRef = useRef<HTMLVideoElement>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const streamRef = useRef<MediaStream | null>(null);

  // Create room and get access token
  const createRoom = async () => {
    try {
      const config: RoomConfig = {
        video_codec: "h264",
        audio_codec: "opus",
        max_bitrate: 2000000,
        frame_rate: 30,
        resolution: "1280x720",
        deduplication_enabled: true,
        adaptive_bitrate: true,
        enable_frame_batching: true,
        enable_hardware_acceleration: true,
      };

      const response = await axios.post("http://localhost:3000/room", {
        api_key: apiKey,
        config,
      });

      setRoomId(response.data.room_id);
      return response.data.access_token;
    } catch (error) {
      onError(error as Error);
      return null;
    }
  };

  // Initialize WebSocket connection
  const initializeWebSocket = (accessToken: string) => {
    if (!roomId) return;

    const ws = new WebSocket(`ws://localhost:3000/room/${roomId}/ws`);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log("WebSocket connected");
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.type === "stats") {
        setStats(data.stats);
      }
    };

    ws.onerror = (error) => {
      onError(error as unknown as Error);
    };
  };

  // Initialize media stream
  const initializeStream = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: {
          width: 1280,
          height: 720,
          frameRate: 30,
        },
        audio: true,
      });

      streamRef.current = stream;
      if (videoRef.current) {
        videoRef.current.srcObject = stream;
      }

      // Initialize MediaRecorder
      const mediaRecorder = new MediaRecorder(stream, {
        mimeType: "video/webm;codecs=h264,opus",
      });

      mediaRecorderRef.current = mediaRecorder;

      mediaRecorder.ondataavailable = async (event) => {
        if (
          event.data.size > 0 &&
          wsRef.current?.readyState === WebSocket.OPEN
        ) {
          const buffer = await event.data.arrayBuffer();
          wsRef.current.send(
            JSON.stringify({
              type: "Frame",
              timestamp: Date.now(),
              data: Array.from(new Uint8Array(buffer)),
              frame_type: "Video",
            })
          );
        }
      };

      // Collect data every 100ms
      mediaRecorder.start(100);
    } catch (error) {
      onError(error as Error);
    }
  };

  const startRecording = async () => {
    const accessToken = await createRoom();
    if (!accessToken) return;

    initializeWebSocket(accessToken);
    await initializeStream();

    if (wsRef.current) {
      wsRef.current.send(
        JSON.stringify({
          type: "Control",
          action: "StartRecording",
        })
      );
    }

    setIsRecording(true);
  };

  const stopRecording = () => {
    if (wsRef.current) {
      wsRef.current.send(
        JSON.stringify({
          type: "Control",
          action: "StopRecording",
        })
      );
      wsRef.current.close();
    }

    if (mediaRecorderRef.current) {
      mediaRecorderRef.current.stop();
    }

    if (streamRef.current) {
      streamRef.current.getTracks().forEach((track) => track.stop());
    }

    setIsRecording(false);
  };

  useEffect(() => {
    return () => {
      if (isRecording) {
        stopRecording();
      }
    };
  }, []);

  return (
    <div className="stream-recorder">
      <div className="video-container">
        <video
          ref={videoRef}
          autoPlay
          playsInline
          muted
          className="preview-video"
        />
      </div>

      <div className="controls">
        <button
          onClick={isRecording ? stopRecording : startRecording}
          className={`record-button ${isRecording ? "recording" : ""}`}
        >
          {isRecording ? "Stop Recording" : "Start Recording"}
        </button>
      </div>

      <div className="stats">
        <div>FPS: {stats.fps}</div>
        <div>Bitrate: {(stats.bitrate / 1000000).toFixed(2)} Mbps</div>
        <div>Frames: {stats.frameCount}</div>
        <div>Deduplication: {(stats.deduplicationRatio * 100).toFixed(1)}%</div>
      </div>

      <style>{`
        .stream-recorder {
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: 20px;
          background: #f5f5f5;
          border-radius: 8px;
        }

        .video-container {
          width: 100%;
          max-width: 800px;
          margin-bottom: 20px;
        }

        .preview-video {
          width: 100%;
          border-radius: 4px;
          background: #000;
        }

        .controls {
          margin-bottom: 20px;
        }

        .record-button {
          padding: 12px 24px;
          font-size: 16px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          background: #2196f3;
          color: white;
          transition: all 0.3s ease;
        }

        .record-button.recording {
          background: #f44336;
        }

        .stats {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: 10px;
          padding: 15px;
          background: white;
          border-radius: 4px;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .stats div {
          font-size: 14px;
          color: #666;
        }
      `}</style>
    </div>
  );
};
