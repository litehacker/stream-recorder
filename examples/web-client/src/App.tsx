import React, { useState } from "react";
import { ApiClient, Room as RoomType } from "./api";
import { Room } from "./components/Room";
import { RoomList } from "./components/RoomList";
import { Auth } from "./components/Auth";

const App: React.FC = () => {
  const [apiClient, setApiClient] = useState<ApiClient | null>(null);
  const [selectedRoom, setSelectedRoom] = useState<RoomType | null>(null);

  const handleAuthenticated = (client: ApiClient) => {
    setApiClient(client);
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
          <Auth onAuthenticated={handleAuthenticated} />
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
