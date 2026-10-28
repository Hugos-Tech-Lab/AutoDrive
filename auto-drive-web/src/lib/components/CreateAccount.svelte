<script lang="ts">
  import { supabase } from "$lib/supabaseClient";

  let loading = $state(false);

  let loginEmail = $state("");
  let loginPassword = $state("");

  let signupEmail = $state("");
  let signupPassword = $state("");

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

    <!-- Sign Up -->
    <form
      class="form-widget m-4 p-4"
      onsubmit={(e) => {
        e.preventDefault();
        handleSignup();
      }}
    >
      <h2 class="description">Create Account</h2>

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