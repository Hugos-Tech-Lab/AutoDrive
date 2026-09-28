<script lang="ts">
  import { supabase } from "$lib/supabaseClient";

  let loading = $state(false);

  let magicEmail = $state("");
  let loginEmail = $state("");
  let loginPassword = $state("");

  let signupEmail = $state("");
  let signupPassword = $state("");

  const handleMagicLink = async () => {
    try {
      loading = true;

      const { error } = await supabase.auth.signInWithOtp({
        email: magicEmail
      });

      if (error) throw error;

      alert("Check your email for the login link!");
    } catch (error) {
      alert(error instanceof Error ? error.message : "Something went wrong.");
    } finally {
      loading = false;
    }
  };

  const handlePasswordLogin = async () => {
    try {
      loading = true;

      const { error } = await supabase.auth.signInWithPassword({
        email: loginEmail,
        password: loginPassword
      });

      if (error) throw error;

      // Supabase has authenticated the user.
      // Add navigation here if required.
    } catch (error) {
      alert(error instanceof Error ? error.message : "Something went wrong.");
    } finally {
      loading = false;
    }
  };

  const handleSignup = async () => {
    try {
      loading = true;

      const { error } = await supabase.auth.signUp({
        email: signupEmail,
        password: signupPassword
      });

      if (error) throw error;

      alert("Account created! Check your email if confirmation is required.");
    } catch (error) {
      alert(error instanceof Error ? error.message : "Something went wrong.");
    } finally {
      loading = false;
    }
  };
</script>

<div class="row flex-center flex">
  <div class="col-6 form-widget" aria-live="polite">
    <!-- Magic Link -->
    <form
      class="form-widget m-4 p-4"
      onsubmit={(e) => {
        e.preventDefault();
        handleMagicLink();
      }}
    >
      <p class="description">
        Sign in via a magic link sent to your email.
      </p>

      <div>
        <label for="magic-email">Email</label>
        <input
          id="magic-email"
          class="inputField w-80"
          type="email"
          placeholder="Your email"
          bind:value={magicEmail}
          required
        />
      </div>

      <div>
        <button
          type="submit"
          class="button block"
          disabled={loading}
        >
          <span>{loading ? "Loading..." : "Send magic link"}</span>
        </button>
      </div>
    </form>

    <!-- Password Login -->
    <form
      class="form-widget m-4 p-4"
      onsubmit={(e) => {
        e.preventDefault();
        handlePasswordLogin();
      }}
    >
      <h2 class="description">Sign in via password</h2>

      <div>
        <label for="login-email">Email</label>
        <input
          id="login-email"
          class="inputField"
          type="email"
          placeholder="Your email"
          bind:value={loginEmail}
          required
        />
      </div>

      <div>
        <label for="login-password">Password</label>
        <input
          id="login-password"
          class="inputField"
          type="password"
          placeholder="Your password"
          bind:value={loginPassword}
          required
        />
      </div>

      <div>
        <button
          type="submit"
          class="button block"
          disabled={loading}
        >
          <span>{loading ? "Loading..." : "Sign in"}</span>
        </button>
      </div>
    </form>

    <!-- Sign Up -->
    <form
      class="form-widget m-4 p-4"
      onsubmit={(e) => {
        e.preventDefault();
        handleSignup();
      }}
    >
      <h2 class="description">Sign up</h2>

      <div>
        <label for="signup-email">Email</label>
        <input
          id="signup-email"
          class="inputField"
          type="email"
          placeholder="Your email"
          bind:value={signupEmail}
          required
        />
      </div>

      <div>
        <label for="signup-password">Password</label>
        <input
          id="signup-password"
          class="inputField"
          type="password"
          placeholder="Your password"
          bind:value={signupPassword}
          minlength="6"
          required
        />
      </div>

      <div>
        <button
          type="submit"
          class="button block"
          disabled={loading}
        >
          <span>{loading ? "Loading..." : "Create account"}</span>
        </button>
      </div>
    </form>
  </div>
</div>