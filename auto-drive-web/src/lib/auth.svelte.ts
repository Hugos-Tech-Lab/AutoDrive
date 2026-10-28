import { supabase } from '$lib/supabaseClient';
import { getUsername } from '$lib/getUsername';
import type { AuthSession } from '@supabase/supabase-js';
import { jwtDecode } from 'jwt-decode';

export type AuthState =
  | { status: 'LOADING'; session: null; username: null; role: null }
  | { status: 'GUEST'; session: null; username: null; role: null }
  | { status: 'AUTHENTICATED'; session: AuthSession; username: string | null; role: string | null };

export function createAuthState() {
  let state = $state<AuthState>({
    status: 'LOADING',
    session: null,
    username: null,
    role: null
  });

  async function updateAuth(session: AuthSession | null) {
    if (!session) {
      state = { status: 'GUEST', session: null, username: null, role: null };
      return;
    }

    const jwt = jwtDecode<{ user_role?: string }>(session.access_token);
    const role = jwt.user_role ?? null;
    console.log(role);

    const username = await getUsername(session.user);
    state = { status: 'AUTHENTICATED', session, username, role };
  }

  supabase.auth.getSession().then(({ data }) => {
    updateAuth(data.session);
  });

  const { data: { subscription } } = supabase.auth.onAuthStateChange((_event, session) => {
    updateAuth(session);
  });

  return {
    get current() { return state; },
    destroy: () => subscription.unsubscribe()
  };
}