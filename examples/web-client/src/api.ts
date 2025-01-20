export interface Room {
  id: string;
  name: string;
  max_participants: number;
  recording_enabled: boolean;
  current_participants: number;
  start_time: string;
  end_time: string | null;
}

export interface CreateRoomRequest {
  name: string;
  max_participants: number;
  recording_enabled?: boolean;
}

export interface ApiResponse<T> {
  data: T;
  error?: string;
}

export class ApiClient {
  private apiKey: string;
  private baseUrl: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
    this.baseUrl = process.env.REACT_APP_API_URL || "http://localhost:3000";
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${this.apiKey}`,
        ...options.headers,
      },
    });

    const data = await response.json();
    if (!response.ok) {
      throw new Error(data.error || "An error occurred");
    }

    return data;
  }

  async generateCredentials(): Promise<ApiResponse<{ token: string }>> {
    return this.request<{ token: string }>("/api/auth/credentials", {
      method: "POST",
    });
  }

  async createRoom(request: CreateRoomRequest): Promise<ApiResponse<Room>> {
    return this.request<Room>("/api/rooms", {
      method: "POST",
      body: JSON.stringify(request),
    });
  }

  async listRooms(): Promise<ApiResponse<Room[]>> {
    return this.request<Room[]>("/api/rooms");
  }

  async getRoomRecordings(roomId: string): Promise<ApiResponse<string[]>> {
    return this.request<string[]>(`/api/rooms/${roomId}/recordings`);
  }
}
