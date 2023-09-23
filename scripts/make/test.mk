# Test scripts

define unit_test
  $(call run_cmd,cargo test,-p percpu $(1) -- --nocapture)
  $(call run_cmd,cargo test,-p axfs $(1) --features "myfs" --test test_ramfs -- --nocapture)
  $(call run_cmd,cargo test,-p axfs $(1) --features "fatfs" --test test_fatfs -- --nocapture)
  $(call run_cmd,cargo test,-p axfs $(1) --features "diskfs" --test test_diskfs -- --nocapture)
  $(call run_cmd,cargo test,--workspace --exclude "arceos-*" $(1) -- --nocapture)
endef

test_app :=
ifneq ($(filter command line,$(origin A) $(origin APP)),)
  test_app := $(APP)
endif

define app_test
  $(CURDIR)/scripts/test/app_test.sh $(test_app)
endef
