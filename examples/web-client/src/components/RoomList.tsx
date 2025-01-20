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
      setLoading(true);
      const response = await apiClient.listRooms();
      const validRooms = response.data.filter(
        (room) => room && room.id && room.name
      );
      setRooms(validRooms);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load rooms");
      setRooms([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadRooms();
  }, [apiClient]);

  const handleCreateRoom = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      setLoading(true);
      const response = await apiClient.createRoom(newRoom);
      console.log("Room creation response:", response.data); // Debug log

      if (response.data) {
        setRooms((prevRooms) => [...prevRooms, response.data]);
        setShowCreateForm(false);
        setNewRoom({ name: "", max_participants: 10, recording_enabled: true });
        setError(null);
      } else {
        console.error("Empty response data");
        throw new Error("Failed to create room: No data received");
      }
    } catch (err) {
      console.error("Room creation error:", err);
      setError(err instanceof Error ? err.message : "Failed to create room");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-6xl mx-auto mt-10 p-6 bg-white rounded-lg shadow-lg">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">Available Rooms</h2>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700"
        >
          {showCreateForm ? "Cancel" : "Create Room"}
        </button>
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
