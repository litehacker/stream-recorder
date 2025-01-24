import React, { useEffect, useState } from "react";
import { Room, ApiClient, CreateRoomRequest } from "../api";

interface RoomListProps {
  apiClient: ApiClient;
  onRoomSelect: (room: Room) => void;
}

export const RoomList: React.FC<RoomListProps> = ({
  apiClient,
  onRoomSelect,
}) => {
  const [rooms, setRooms] = useState<Room[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newRoom, setNewRoom] = useState<CreateRoomRequest>({
    name: "",
    max_participants: 10,
    recording_enabled: true,
  });

  const loadRooms = async () => {
    try {
      // Check for token before making request
      const token = localStorage.getItem("jwt_token");
      if (!token) {
        setError("No authentication token found. Please log in again.");
        setRooms([]);
        return;
      }

      setLoading(true);
      console.log("Fetching rooms with token:", token.substring(0, 10) + "...");
      const response = await apiClient.listRooms();
      console.log("Rooms response:", response);

      // Handle direct array response
      const rooms = response.data || response;
      console.log("Parsed rooms:", rooms);

      const validRooms = rooms.filter((room) => room && room.id && room.name);
      console.log("Valid rooms:", validRooms);

      setRooms(validRooms);
      setError(null);
    } catch (err) {
      console.error("Error loading rooms:", err);
      let errorMessage = "Failed to load rooms";

      // Handle specific error cases
      if (err instanceof Error) {
        if (
          err.message.includes("401") ||
          err.message.includes("unauthorized")
        ) {
          errorMessage = "Authentication failed. Please log in again.";
        } else if (err.message.includes("403")) {
          errorMessage = "You don't have permission to view rooms.";
        } else {
          errorMessage = err.message;
        }
      }

      setError(errorMessage);
      setRooms([]);
    } finally {
      setLoading(false);
    }
  };

  // Add manual refresh capability
  const handleRefresh = () => {
    loadRooms();
  };

  useEffect(() => {
    loadRooms();
  }, [apiClient]);

  const handleCreateRoom = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      // Check for token before making request
      const token = localStorage.getItem("jwt_token");
      if (!token) {
        setError("No authentication token found. Please log in again.");
        return;
      }

      setLoading(true);
      setError(null);

      console.log("Creating room with data:", newRoom);
      const response = await apiClient.createRoom(newRoom);
      console.log("Room creation response:", response);

      // Handle direct room response
      const room = response.data || response;
      if (!room || !room.id || !room.name) {
        console.error("Invalid room data:", room);
        throw new Error("Server returned invalid room data");
      }

      setRooms((prevRooms) => [...prevRooms, room]);
      setShowCreateForm(false);
      setNewRoom({ name: "", max_participants: 10, recording_enabled: true });
    } catch (err) {
      console.error("Room creation error:", err);
      let errorMessage = "Failed to create room";

      // Handle specific error cases
      if (err instanceof Error) {
        if (
          err.message.includes("401") ||
          err.message.includes("unauthorized")
        ) {
          errorMessage = "Authentication failed. Please log in again.";
        } else if (err.message.includes("403")) {
          errorMessage = "You don't have permission to create rooms.";
        } else {
          errorMessage = `Failed to create room: ${err.message}`;
        }
      }

      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-6xl mx-auto mt-10 p-6 bg-white rounded-lg shadow-lg">
      <div className="flex justify-between items-center mb-6">
        {showCreateForm ? (
          <div className="flex items-center space-x-4">
            <button
              onClick={() => setShowCreateForm(false)}
              className="bg-gray-100 text-gray-600 px-3 py-2 rounded-md hover:bg-gray-200 flex items-center"
            >
              <span className="mr-1">←</span> Back
            </button>
            <h2 className="text-2xl font-bold">Create New Room</h2>
          </div>
        ) : (
          <div className="flex items-center space-x-4">
            <h2 className="text-2xl font-bold">Available Rooms</h2>
            <button
              onClick={handleRefresh}
              className="bg-gray-100 text-gray-600 px-3 py-2 rounded-md hover:bg-gray-200 flex items-center"
              title="Refresh room list"
            >
              🔄 Refresh
            </button>
          </div>
        )}
        {!showCreateForm && (
          <button
            onClick={() => setShowCreateForm(true)}
            className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700"
          >
            Create Room
          </button>
        )}
      </div>

      {error && (
        <div className="mb-4 p-4 bg-red-100 text-red-700 rounded-md">
          {error}
        </div>
      )}

      {showCreateForm && (
        <form onSubmit={handleCreateRoom} className="mb-8 space-y-4">
          <div className="mb-4">
            <label
              htmlFor="roomName"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Room Name
            </label>
            <input
              type="text"
              id="roomName"
              value={newRoom.name}
              onChange={(e) => setNewRoom({ ...newRoom, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
              placeholder="Enter room name"
              required
            />
          </div>
          <div className="mb-4">
            <label
              htmlFor="maxParticipants"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Max Participants
            </label>
            <input
              type="number"
              id="maxParticipants"
              value={newRoom.max_participants}
              onChange={(e) =>
                setNewRoom({
                  ...newRoom,
                  max_participants: parseInt(e.target.value, 10),
                })
              }
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
              placeholder="Enter max participants"
              min="1"
              max="100"
              required
            />
          </div>
          <div className="flex items-center mb-4">
            <input
              type="checkbox"
              id="recordingEnabled"
              checked={newRoom.recording_enabled}
              onChange={(e) =>
                setNewRoom({
                  ...newRoom,
                  recording_enabled: e.target.checked,
                })
              }
              className="h-4 w-4 text-indigo-600 focus:ring-indigo-500 border-gray-300 rounded"
            />
            <label
              htmlFor="recordingEnabled"
              className="ml-2 block text-sm text-gray-900"
            >
              Enable Recording
            </label>
          </div>
          <button
            type="submit"
            className="w-full bg-green-600 text-white px-4 py-2 rounded-md hover:bg-green-700"
            disabled={loading}
          >
            {loading ? "Creating..." : "Create Room"}
          </button>
        </form>
      )}

      {loading && !showCreateForm ? (
        <div className="text-center text-gray-600">Loading rooms...</div>
      ) : (
        <div className="grid gap-4 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
          {rooms.map(
            (room) =>
              room && (
                <div
                  key={room.id}
                  className="border rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer"
                  onClick={() => onRoomSelect(room)}
                >
                  <h3 className="text-lg font-semibold mb-2">{room.name}</h3>
                  <div className="text-sm text-gray-600">
                    <p>
                      Participants: {room.current_participants}/
                      {room.max_participants}
                    </p>
                    <p>
                      Recording:{" "}
                      {room.recording_enabled ? "Enabled" : "Disabled"}
                    </p>
                    <p>Started: {new Date(room.start_time).toLocaleString()}</p>
                    {room.end_time && (
                      <p>Ended: {new Date(room.end_time).toLocaleString()}</p>
                    )}
                  </div>
                </div>
              )
          )}
        </div>
      )}

      {!loading && rooms.length === 0 && (
        <div className="text-center text-gray-600">
          {error
            ? `Error: ${error}`
            : "No rooms available. Create one to get started!"}
        </div>
      )}
    </div>
  );
};
