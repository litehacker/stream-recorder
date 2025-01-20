import React, { useState, useEffect } from "react";
import { ApiClient } from "../api";

interface AuthProps {
  onAuthenticated: (client: ApiClient) => void;
}

export const Auth: React.FC<AuthProps> = ({ onAuthenticated }) => {
  const [apiKey, setApiKey] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [loadingMessage, setLoadingMessage] = useState("");

  // Auto-connect when API key is generated
  useEffect(() => {
    if (apiKey && loadingMessage === "Generated new API key") {
      handleSubmit(null);
    }
  }, [apiKey]);

  const handleSubmit = async (e: React.FormEvent | null) => {
    e?.preventDefault();
    if (!apiKey.trim()) {
      setError("Please enter an API key");
      return;
    }

    setLoading(true);
    setLoadingMessage("Connecting to server");
    setError("");

    try {
      const client = new ApiClient(apiKey);
      await client.generateCredentials();
      onAuthenticated(client);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to authenticate");
      // Clear API key if it's invalid
      if (err instanceof Error && err.message.includes("invalid")) {
        setApiKey("");
      }
    } finally {
      setLoading(false);
      setLoadingMessage("");
    }
  };

  const handleGenerate = async () => {
    setLoading(true);
    setLoadingMessage("Generating new API key");
    setError("");

    try {
      const tempClient = new ApiClient("");
      const response = await tempClient.generateCredentials();
      setApiKey(response.data.token);
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to generate API key"
      );
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      handleSubmit(null);
    }
  };

  return (
    <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-lg">
      <h2 className="text-2xl font-bold mb-6">Authentication</h2>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label
            htmlFor="apiKey"
            className="block text-sm font-medium text-gray-700"
          >
            API Key
          </label>
          <div className="mt-1 relative rounded-md shadow-sm">
            <input
              id="apiKey"
              type="password"
              value={apiKey}
              onChange={(e) => {
                setApiKey(e.target.value);
                if (error) setError("");
              }}
              onKeyDown={handleKeyPress}
              className="block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:border-indigo-500 focus:ring-indigo-500 disabled:bg-gray-100 disabled:cursor-not-allowed"
              placeholder="Enter your API key"
              disabled={loading}
              aria-invalid={error ? "true" : "false"}
              aria-describedby={error ? "auth-error" : undefined}
            />
          </div>
        </div>

        {error && (
          <div
            id="auth-error"
            role="alert"
            className="text-red-600 text-sm bg-red-50 p-3 rounded border border-red-200"
          >
            {error}
          </div>
        )}

        {loading && (
          <div className="text-indigo-600 text-sm bg-indigo-50 p-3 rounded border border-indigo-200">
            {loadingMessage}...
          </div>
        )}

        <div className="flex space-x-4">
          <button
            type="submit"
            disabled={loading}
            className="flex-1 bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200"
          >
            {loading ? "Connecting..." : "Connect"}
          </button>
          <button
            type="button"
            onClick={handleGenerate}
            disabled={loading}
            className="flex-1 bg-gray-600 text-white px-4 py-2 rounded-md hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200"
          >
            {loading ? "Generating..." : "Generate New"}
          </button>
        </div>
      </form>
    </div>
  );
};
