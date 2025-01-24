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
    const url = `${this.baseUrl}${endpoint}`;
    console.log(`Making request to: ${url}`, { method: options.method });

    const response = await fetch(url, {
      ...options,
      credentials: "include", // This is required for cookies
      headers: {
        "Content-Type": "application/json",
        ...(endpoint === "/api/auth/credentials"
          ? {
              Authorization: `Bearer ${this.apiKey}`,
            }
          : {}),
        ...options.headers,
      },
    });

    const contentType = response.headers.get("content-type");
    if (!contentType || !contentType.includes("application/json")) {
      console.error("Non-JSON response received:", { contentType });
      throw new Error("Invalid response format");
    }

    let text;
    try {
      text = await response.text();
      console.log("Raw response:", text);
    } catch (e) {
      console.error("Failed to read response:", e);
      throw new Error("Failed to read response");
    }

    if (!text) {
      throw new Error("Empty response received");
    }

    let parsed;
    try {
      parsed = JSON.parse(text);
    } catch (e) {
      console.error("Failed to parse JSON:", e);
      throw new Error("Invalid JSON response");
    }

    if (!response.ok) {
      throw new Error(parsed?.error || "An error occurred");
    }

    // If the response is already in the format we want, use it directly
    if (parsed.data) {
      return parsed as ApiResponse<T>;
    }

    // Otherwise, wrap the response in our ApiResponse format
    return {
      data: parsed as T,
      error: undefined,
    };
  }

  async generateCredentials(): Promise<ApiResponse<{ message: string }>> {
    return this.request<{ message: string }>("/api/auth/credentials", {
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
    console.log("Calling listRooms endpoint...");
    const response = await this.request<Room[]>("/api/rooms");
    console.log("ListRooms raw response:", response);
    return response;
  }

  async getRoomRecordings(roomId: string): Promise<ApiResponse<string[]>> {
    return this.request<string[]>(`/api/rooms/${roomId}/recordings`);
  }
}
