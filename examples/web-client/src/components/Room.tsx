import React, { useEffect, useRef, useState, useCallback } from "react";
import { Room as RoomType } from "../api";

type WebSocketWithRef = WebSocket & {
  wasManuallyClosedRef?: { current: boolean };
};

interface RoomProps {
  room: RoomType;
  onLeave: () => void;
}

export const Room: React.FC<RoomProps> = ({ room, onLeave }) => {
  const [error, setError] = useState("");
  const [isConnected, setIsConnected] = useState(false);
  const [isReconnecting, setIsReconnecting] = useState(false);
  const [localStream, setLocalStream] = useState<MediaStream | null>(null);
  const [remoteStreams, setRemoteStreams] = useState<MediaStream[]>([]);
  const [isMuted, setIsMuted] = useState(false);
  const [isVideoEnabled, setIsVideoEnabled] = useState(true);

  const wsRef = useRef<WebSocketWithRef | null>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout>();
  const reconnectAttemptsRef = useRef(0);
  const MAX_RECONNECT_ATTEMPTS = 3;

  const setupMediaStream = useCallback(async () => {
    try {
      // First check if permissions are already granted
      const permissions = await navigator.mediaDevices.enumerateDevices();
      const hasVideoPermission = permissions.some(
        (device) => device.kind === "videoinput" && device.label
      );
      const hasAudioPermission = permissions.some(
        (device) => device.kind === "audioinput" && device.label
      );

      if (!hasVideoPermission || !hasAudioPermission) {
        setError("Please allow camera and microphone access to join the room");
        // Show a button to request permissions
        return null;
      }

      const stream = await navigator.mediaDevices.getUserMedia({
        video: {
          width: { ideal: 1280 },
          height: { ideal: 720 },
          frameRate: { ideal: 30 },
        },
        audio: true,
      });
      setLocalStream(stream);

      if (localVideoRef.current) {
        localVideoRef.current.srcObject = stream;
      }
      return stream;
    } catch (err) {
      if (err instanceof Error) {
        if (
          err.name === "NotAllowedError" ||
          err.name === "PermissionDeniedError"
        ) {
          setError(
            "Camera and microphone access was denied. Please allow access in your browser settings and try again."
          );
        } else if (
          err.name === "NotFoundError" ||
          err.name === "DevicesNotFoundError"
        ) {
          setError(
            "No camera or microphone found. Please connect a device and try again."
          );
        } else if (
          err.name === "NotReadableError" ||
          err.name === "TrackStartError"
        ) {
          setError(
            "Your camera or microphone is already in use by another application."
          );
        } else {
          setError(`Failed to access media devices: ${err.message}`);
        }
      } else {
        setError("An unknown error occurred while accessing media devices");
      }
      return null;
    }
  }, []);

  const connectWebSocket = useCallback(
    (stream: MediaStream) => {
      if (!process.env.REACT_APP_WS_URL) {
        setError("WebSocket URL is not configured");
        return;
      }

      // Get the JWT token from localStorage
      const token = localStorage.getItem("jwt_token");
      if (!token) {
        setError("No authentication token found");
        return;
      }

      const ws = new WebSocket(
        `${process.env.REACT_APP_WS_URL}/api/rooms/${room.id}/ws?token=${token}`
      ) as WebSocketWithRef;
      ws.wasManuallyClosedRef = { current: false };
      wsRef.current = ws;

      ws.onopen = () => {
        setIsConnected(true);
        setIsReconnecting(false);
        reconnectAttemptsRef.current = 0;
        console.log("Connected to room");
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          // Handle different message types
          switch (data.type) {
            case "participant_joined":
              console.log("New participant joined:", data.participantId);
              // Add remote stream for new participant
              if (data.stream) {
                setRemoteStreams((streams) => [
                  ...streams,
                  new MediaStream(data.stream),
                ]);
              }
              break;
            case "participant_left":
              console.log("Participant left:", data.participantId);
              // Remove remote stream for participant
              if (data.participantId) {
                setRemoteStreams((streams) =>
                  streams.filter((_, index) => index !== data.participantId)
                );
              }
              break;
            case "stream":
              // Handle incoming media stream
              if (data.stream) {
                const mediaStream = new MediaStream(data.stream);
                setRemoteStreams((streams) => [...streams, mediaStream]);
              }
              break;
            default:
              console.log("Received message:", data);
          }
        } catch (err) {
          console.error("Failed to parse message:", err);
        }
      };

      ws.onerror = (error) => {
        console.error("WebSocket error:", error);
        setError("Connection error occurred");
      };

      ws.onclose = () => {
        setIsConnected(false);
        console.log("Disconnected from room");

        // Attempt to reconnect if not manually closed
        if (
          !ws.wasManuallyClosedRef?.current &&
          reconnectAttemptsRef.current < MAX_RECONNECT_ATTEMPTS
        ) {
          setIsReconnecting(true);
          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectAttemptsRef.current++;
            connectToRoom();
          }, 2000 * Math.pow(2, reconnectAttemptsRef.current));
        }
      };
    },
    [room.id]
  );

  const connectToRoom = useCallback(async () => {
    try {
      const stream = await setupMediaStream();
      if (stream) {
        connectWebSocket(stream);
      }
    } catch (err) {
      setError(
        `Failed to connect to room: ${
          err instanceof Error ? err.message : "Unknown error"
        }`
      );
      setIsConnected(false);
    }
  }, [setupMediaStream, connectWebSocket]);

  useEffect(() => {
    connectToRoom();

    return () => {
      // Cleanup
      if (wsRef.current?.wasManuallyClosedRef) {
        wsRef.current.wasManuallyClosedRef.current = true;
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (localStream) {
        localStream.getTracks().forEach((track) => track.stop());
      }
    };
  }, [connectToRoom]);

  const handleLeave = () => {
    if (wsRef.current?.wasManuallyClosedRef) {
      wsRef.current.wasManuallyClosedRef.current = true;
    }
    if (wsRef.current) {
      wsRef.current.close();
    }
    if (localStream) {
      localStream.getTracks().forEach((track) => track.stop());
    }
    onLeave();
  };

  const toggleAudio = () => {
    if (localStream) {
      const audioTrack = localStream.getAudioTracks()[0];
      if (audioTrack) {
        audioTrack.enabled = !audioTrack.enabled;
        setIsMuted(!audioTrack.enabled);
      }
    }
  };

  const toggleVideo = () => {
    if (localStream) {
      const videoTrack = localStream.getVideoTracks()[0];
      if (videoTrack) {
        videoTrack.enabled = !videoTrack.enabled;
        setIsVideoEnabled(!isVideoEnabled);
      }
    }
  };

  // Add a function to request permissions explicitly
  const requestMediaPermissions = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: true,
        audio: true,
      });
      // Stop the stream immediately as we just want to trigger the permission prompt
      stream.getTracks().forEach((track) => track.stop());
      // Try to set up the media stream again
      connectToRoom();
    } catch (err) {
      if (err instanceof Error) {
        setError(`Failed to get permissions: ${err.message}`);
      }
    }
  };

  return (
    <div className="max-w-6xl mx-auto mt-10 p-6 bg-white rounded-lg shadow-lg">
      <div className="flex justify-between items-center mb-6">
        <div className="flex items-center space-x-4">
          <button
            onClick={onLeave}
            className="bg-gray-100 text-gray-600 px-3 py-2 rounded-md hover:bg-gray-200 flex items-center"
          >
            <span className="mr-1">←</span> Back
          </button>
          <div>
            <h2 className="text-2xl font-bold">{room.name}</h2>
            <p className="text-sm text-gray-500">Room ID: {room.id}</p>
          </div>
        </div>
        <div className="flex items-center space-x-4">
          <div className="flex items-center">
            <span
              className={`inline-block w-3 h-3 rounded-full ${
                isConnected
                  ? "bg-green-500"
                  : isReconnecting
                  ? "bg-yellow-500"
                  : "bg-red-500"
              }`}
            />
            <span className="ml-2 text-sm text-gray-600">
              {isConnected
                ? "Connected"
                : isReconnecting
                ? "Reconnecting..."
                : "Disconnected"}
            </span>
          </div>
          <div className="flex space-x-2">
            <button
              onClick={toggleAudio}
              className={`p-2 rounded-full ${
                isMuted
                  ? "bg-red-100 text-red-600"
                  : "bg-gray-100 text-gray-600"
              } hover:bg-gray-200`}
              title={isMuted ? "Unmute" : "Mute"}
            >
              {isMuted ? "🔇" : "🎤"}
            </button>
            <button
              onClick={toggleVideo}
              className={`p-2 rounded-full ${
                !isVideoEnabled
                  ? "bg-red-100 text-red-600"
                  : "bg-gray-100 text-gray-600"
              } hover:bg-gray-200`}
              title={isVideoEnabled ? "Turn off camera" : "Turn on camera"}
            >
              {isVideoEnabled ? "📹" : "🚫"}
            </button>
            <button
              onClick={handleLeave}
              className="bg-red-600 text-white px-4 py-2 rounded-md hover:bg-red-700 transition-colors"
            >
              Leave Room
            </button>
          </div>
        </div>
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-100 text-red-700 rounded-md border border-red-200">
          {error}
          {(error.includes("allow") || error.includes("denied")) && (
            <button
              onClick={requestMediaPermissions}
              className="ml-4 bg-red-600 text-white px-4 py-2 rounded-md hover:bg-red-700 transition-colors"
            >
              Request Permissions
            </button>
          )}
        </div>
      )}

      <div className="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
        {/* Local video */}
        <div className="relative">
          <video
            ref={localVideoRef}
            autoPlay
            playsInline
            muted
            className={`w-full rounded-lg ${
              !isVideoEnabled ? "bg-gray-900" : ""
            }`}
          />
          <div className="absolute bottom-2 left-2 bg-black bg-opacity-50 text-white px-2 py-1 rounded">
            You {isMuted && "🔇"} {!isVideoEnabled && "🚫"}
          </div>
        </div>

        {/* Remote videos */}
        {remoteStreams.map((stream, index) => (
          <div key={index} className="relative">
            <video
              autoPlay
              playsInline
              className="w-full rounded-lg"
              ref={(video) => {
                if (video) video.srcObject = stream;
              }}
            />
            <div className="absolute bottom-2 left-2 bg-black bg-opacity-50 text-white px-2 py-1 rounded">
              Participant {index + 1}
            </div>
          </div>
        ))}

        {/* Placeholder for empty slots */}
        {Array.from({
          length: Math.max(0, room.max_participants - remoteStreams.length - 1),
        }).map((_, index) => (
          <div
            key={`empty-${index}`}
            className="w-full aspect-video bg-gray-100 rounded-lg flex items-center justify-center"
          >
            <span className="text-gray-400">Empty Slot</span>
          </div>
        ))}
      </div>
    </div>
  );
};
