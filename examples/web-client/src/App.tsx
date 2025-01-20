import React, { useState } from "react";
import { ApiClient, Room as RoomType } from "./api";
import { Room } from "./components/Room";
import { RoomList } from "./components/RoomList";

const App: React.FC = () => {
  const [apiKey, setApiKey] = useState("");
  const [apiClient, setApiClient] = useState<ApiClient | null>(null);
  const [selectedRoom, setSelectedRoom] = useState<RoomType | null>(null);

  const handleApiKeySubmit = async (key: string) => {
    const client = new ApiClient(key);
    try {
      await client.generateCredentials(); // Validate API key
      setApiClient(client);
      setApiKey(key);
    } catch (error) {
      alert("Invalid API key");
    }
  };

  const handleRoomSelect = (room: RoomType) => {
    setSelectedRoom(room);
  };

  const handleLeaveRoom = () => {
    setSelectedRoom(null);
  };

  if (!apiClient) {
    return (
      <div className="min-h-screen bg-gray-100 py-12 px-4 sm:px-6 lg:px-8">
        <div className="max-w-md mx-auto">
          <h1 className="text-3xl font-bold text-center mb-8">
            Stream Recorder
          </h1>
          <div className="bg-white p-6 rounded-lg shadow-md">
            <label
              htmlFor="apiKey"
              className="block text-sm font-medium text-gray-700 mb-2"
            >
              Enter API Key
            </label>
            <input
              type="password"
              id="apiKey"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500"
              placeholder="Your API key"
            />
            <button
              onClick={() => handleApiKeySubmit(apiKey)}
              className="w-full mt-4 bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700"
            >
              Connect
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (selectedRoom) {
    return <Room room={selectedRoom} onLeave={handleLeaveRoom} />;
  }

  return <RoomList apiClient={apiClient} onRoomSelect={handleRoomSelect} />;
};

export default App;
