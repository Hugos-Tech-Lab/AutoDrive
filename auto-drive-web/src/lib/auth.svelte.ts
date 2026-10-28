import { supabase } from '$lib/supabaseClient';
import { getUsername } from '$lib/getUsername';
import type { AuthSession } from '@supabase/supabase-js';

export type AuthState =
  | { status: 'LOADING'; session: null; username: null }
  | { status: 'GUEST'; session: null; username: null }
  | { status: 'AUTHENTICATED'; session: AuthSession; username: string | null };

export function createAuthState() {
  let state = $state<AuthState>({
    status: 'LOADING',
    session: null,
    username: null
  });

  async function updateAuth(session: AuthSession | null) {
    if (!session) {
      state = { status: 'GUEST', session: null, username: null };
      return;
    }

    const username = await getUsername(session.user);
    state = { status: 'AUTHENTICATED', session, username };
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
