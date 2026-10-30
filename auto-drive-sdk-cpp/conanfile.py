import os

from conan import ConanFile
from conan.tools.files import copy


class TheBeginnerCarSDK(ConanFile):
    name = "auto-drive-sdk-c"
    version = "0.1.0"
    package_type = "header-library"

    exports_sources = "include/*"

    def package(self):
        copy(
            self,
            "*.h",
            src=os.path.join(self.source_folder, "include"),
            dst=os.path.join(self.package_folder, "include"),
        )

    def package_info(self):
        self.cpp_info.includedirs = ["include"]
        self.cpp_info.libdirs = []
        self.cpp_info.bindirs = []
