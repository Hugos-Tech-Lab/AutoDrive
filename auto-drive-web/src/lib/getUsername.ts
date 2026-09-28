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

    // No cache or cache has expired
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

export const getUsernameFromSupabase = async (user: User) => {
  let username = "";
 
  try {
    const { data, error, status } = await supabase
      .from("profiles")
      .select("username")
      .eq("id", user.id)
      .single();

    if (error && status !== 406) throw error;

    if (data) {
      username = data.username;
    }
  } finally {
    //
  }

  return username;
};