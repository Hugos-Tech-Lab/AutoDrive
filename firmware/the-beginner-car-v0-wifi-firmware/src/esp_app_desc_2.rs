// This is a 2nd implementation of esp_app_desc() but with the mmu_page_size hardcoded to 15.
// Otherwise trying to initialize NVS or calling a function like EspDefaultNvsPartition::take()?;
// would cause a reboot loop.
// TODO: create an issue here: https://github.com/esp-rs/esp-idf-sys/issues (since none already seem to exist)

#[macro_export]
macro_rules! esp_app_desc_2 {
  {} => {
    #[unsafe(no_mangle)]
    #[used]
    #[unsafe(link_section = ".rodata_desc")]
    #[allow(non_upper_case_globals)]
    pub static esp_app_desc: esp_app_desc_t = {
      const fn str_to_cstr_array<const C: usize>(s: &str) -> [::core::ffi::c_char; C] {
          let bytes = s.as_bytes();
          assert!(bytes.len() < C);

          let mut ret: [::core::ffi::c_char; C] = [0; C];
          let mut index = 0;
          while index < bytes.len() {
              ret[index] = bytes[index] as _;
              index += 1;
          }

          ret
      }

      esp_app_desc_t {
          magic_word: ESP_APP_DESC_MAGIC_WORD,
          secure_version: 0,
          reserv1: [0; 2],
          version: str_to_cstr_array(env!("CARGO_PKG_VERSION")),
          project_name: str_to_cstr_array(env!("CARGO_PKG_NAME")),
          #[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
          time:
              str_to_cstr_array(build_time_utc!("%H:%M:%S")),
          #[cfg(all(esp_idf_app_compile_time_date, not(esp_idf_app_reproducible_build)))]
          date:
              str_to_cstr_array(build_time_utc!("%Y-%m-%d"))
          ,
          idf_ver: str_to_cstr_array(const_format::formatcp!(
                    "{}.{}.{}",
                    esp_idf_svc::sys::ESP_IDF_VERSION_MAJOR,
                    esp_idf_svc::sys::ESP_IDF_VERSION_MINOR,
                    esp_idf_svc::sys::ESP_IDF_VERSION_PATCH
                )),
          app_elf_sha256: [0; 32],
          min_efuse_blk_rev_full: CONFIG_ESP_EFUSE_BLOCK_REV_MIN_FULL as _,
          max_efuse_blk_rev_full: CONFIG_ESP_EFUSE_BLOCK_REV_MAX_FULL as _,
          mmu_page_size: 15,
          reserv3: [0; 3],
          reserv2: [0; 18],
      }
    };
  }
}
