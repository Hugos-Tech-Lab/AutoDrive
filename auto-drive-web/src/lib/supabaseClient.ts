import { createClient } from '@supabase/supabase-js'

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL || 'https://zldybjfuxctpqpdbekak.supabase.co'
const supabasePublishableKey = import.meta.env.VITE_SUPABASE_PUBLISHABLE_KEY || 'sb_publishable_HxxjKigzAOHgS_oTfgEQDw_52iMN42G'

export const supabase = createClient(supabaseUrl, supabasePublishableKey)
