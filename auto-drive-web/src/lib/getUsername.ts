import type { User } from "@supabase/supabase-js";
import { supabase } from "./supabaseClient";

export const getUsername = async (user: User) => {
  let username = "";
  try {
    const userId = user.id;
    const storageKey = `username:${userId}`;
    const cacheDuration = 60 * 60 * 1000; // 1 hour

    const cached = localStorage.getItem(storageKey);

    if (cached !== null) {
      const { username: cachedUsername, timestamp } = JSON.parse(cached);

      if (Date.now() - timestamp < cacheDuration) {
        username = cachedUsername;
        return username;
      }
    }

    username = await getUsernameFromSupabase(user);

    if (username !== null) {
      localStorage.setItem(
        storageKey,
        JSON.stringify({
          username,
          timestamp: Date.now(),
        }),
      );
    }
  } finally {
    //
  }

  return username;
};

export const getUsernameFromSupabase = (user: User) => {
  return user.user_metadata?.display_name;
};