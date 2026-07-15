import Foundation
import Darwin
import OSLog

class NativeBridge {
    private let logger = Logger(subsystem: "com.promptos.app", category: "NativeBridge")
    private let handle: UnsafeMutableRawPointer?

    init?() {
        let bundle = Bundle.main
        var dylibPath = bundle.bundleURL
            .appendingPathComponent("Contents/Frameworks/libpromptos_llama.dylib").path
        logger.info("NativeBridge init — looking for dylib")
        if !FileManager.default.fileExists(atPath: dylibPath) {
            dylibPath = bundle.path(forResource: "libpromptos_llama", ofType: "dylib")
                ?? bundle.bundleURL
                    .appendingPathComponent("Contents/MacOS/libpromptos_llama.dylib").path
        }

        guard FileManager.default.fileExists(atPath: dylibPath) else {
            logger.warning("NativeBridge — dylib not found at \(dylibPath, privacy: .public), fallback mode")
            self.handle = nil
            return nil
        }

        guard let h = dlopen(dylibPath, RTLD_NOW) else {
            let err = String(cString: dlerror())
            logger.error("NativeBridge — dlopen failed: \(err, privacy: .public)")
            self.handle = nil
            return nil
        }
        self.handle = h
        logger.info("NativeBridge — dylib loaded successfully from \(dylibPath, privacy: .public)")
    }

    deinit {
        if let h = handle {
            dlclose(h)
        }
    }

    func ensureModelDownloaded() -> String? {
        guard let h = handle else { return nil }

        let sym = dlsym(h, "promptos_llm_download_model")
        let fn = unsafeBitCast(sym, to: DownloadFunc.self)

        var buf = [CChar](repeating: 0, count: 1024)
        let result = fn(&buf, 1024)
        let path = String(cString: buf)
        guard result >= 0, !path.isEmpty else { return nil }
        return path
    }

    func loadModel(at path: String) -> Bool {
        guard let h = handle else { return false }

        let sym = dlsym(h, "promptos_llm_init")
        let fn = unsafeBitCast(sym, to: InitFunc.self)

        let result = path.withCString { ptr in
            fn(ptr)
        }
        return result == 0
    }

    var isModelLoaded: Bool {
        guard let h = handle else {
            logger.debug("NativeBridge — model not loaded (handle is nil)")
            return false
        }

        let sym = dlsym(h, "promptos_llm_is_loaded")
        let fn = unsafeBitCast(sym, to: IsLoadedFunc.self)

        let loaded = fn() == 1
        logger.debug("NativeBridge — model loaded: \(loaded, privacy: .public)")
        return loaded
    }

    func compile(_ input: String) -> String? {
        guard let h = handle else { return nil }

        let sym = dlsym(h, "promptos_llm_compile")
        let fn = unsafeBitCast(sym, to: CompileFunc.self)

        var buf = [CChar](repeating: 0, count: 32768)

        let result = input.withCString { ptr in
            fn(ptr, &buf, 32768)
        }

        guard result >= 0 else { return nil }

        let compiled = String(cString: buf)
        return compiled.isEmpty ? nil : compiled
    }

    func unloadModel() {
        guard let h = handle else { return }

        let sym = dlsym(h, "promptos_llm_unload")
        let fn = unsafeBitCast(sym, to: UnloadFunc.self)
        _ = fn()
    }

    typealias InitFunc = @convention(c) (UnsafePointer<CChar>?) -> Int32
    typealias IsLoadedFunc = @convention(c) () -> Int32
    typealias CompileFunc = @convention(c) (UnsafePointer<CChar>?, UnsafeMutablePointer<CChar>?, Int32) -> Int32
    typealias UnloadFunc = @convention(c) () -> Int32
    typealias DownloadFunc = @convention(c) (UnsafeMutablePointer<CChar>?, Int32) -> Int32
}
