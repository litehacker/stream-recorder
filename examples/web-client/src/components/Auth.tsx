import React, { useState } from "react";
import { ApiClient } from "../api";

interface AuthProps {
  onAuthenticated: (client: ApiClient) => void;
}

export const Auth: React.FC<AuthProps> = ({ onAuthenticated }) => {
  const [apiKey, setApiKey] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);

  const handleSignIn = async () => {
    setLoading(true);
    setError("");

    try {
      // Use provided API key or empty string for auto-generation
      const client = new ApiClient(apiKey);
      await client.generateCredentials();
      onAuthenticated(client);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to authenticate");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-lg">
      <h2 className="text-2xl font-bold mb-6">Welcome to Stream Recorder</h2>

      {error && (
        <div
          role="alert"
          className="text-red-600 text-sm bg-red-50 p-3 rounded border border-red-200 mb-4"
        >
          {error}
        </div>
      )}

      {showApiKey ? (
        <div className="mb-4">
          <label
            htmlFor="apiKey"
            className="block text-sm font-medium text-gray-700 mb-2"
          >
            API Key (Optional)
          </label>
          <input
            id="apiKey"
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            placeholder="Enter your API key"
          />
        </div>
      ) : (
        <button
          onClick={() => setShowApiKey(true)}
          className="w-full mb-4 bg-gray-100 text-gray-700 px-4 py-2 rounded-md hover:bg-gray-200 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 text-sm"
        >
          I have an API key
        </button>
      )}

      <button
        onClick={handleSignIn}
        disabled={loading}
        className="w-full bg-indigo-600 text-white px-4 py-3 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-200 text-lg font-medium"
      >
        {loading ? "Signing in..." : "Sign In"}
      </button>

      <p className="mt-4 text-sm text-gray-600 text-center">
        {showApiKey
          ? "Or just click Sign In to continue without an API key"
          : "Click to sign in and start recording your streams"}
      </p>
    </div>
  );
};
