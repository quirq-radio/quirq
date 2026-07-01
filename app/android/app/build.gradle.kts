import java.util.Properties

plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// Load local.properties so we can read platform-specific paths (WSL2 paths
// needed when this Gradle build runs from Android Studio on Windows).
val localProps = Properties().also { props ->
    rootProject.file("local.properties").takeIf { it.exists() }
        ?.inputStream()?.use { props.load(it) }
}

android {
    namespace = "org.quirq.quirq"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_11.toString()
    }

    defaultConfig {
        applicationId = "org.quirq.quirq"
        minSdk = flutter.minSdkVersion
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("debug")
        }
    }
}

flutter {
    source = "../.."
}

// ── Rust FFI bridge ──────────────────────────────────────────────────────────

val jniLibsDir = file("src/main/jniLibs")
val onWindows = System.getProperty("os.name").lowercase().contains("windows")

val cargoBuild by tasks.registering(Exec::class) {
    description = "Cross-compile quirq-ffi for Android via cargo-ndk"

    if (onWindows) {
        // Android Studio runs Gradle natively on Windows, so cargo is not
        // available. Delegate to WSL2 via wsl.exe instead.
        // Requires these entries in local.properties:
        //   wsl.repo.root  – WSL2 absolute path to the repo root
        //   wsl.android.ndk – WSL2 absolute path to the Android NDK
        val wslRoot = localProps.getProperty("wsl.repo.root")
            ?: error("Add wsl.repo.root=/home/<you>/path/to/quirq to local.properties")
        val wslNdk = localProps.getProperty("wsl.android.ndk")
            ?: error("Add wsl.android.ndk=/home/<you>/Android/Sdk/ndk/<ver> to local.properties")
        val wslOut = "$wslRoot/app/android/app/src/main/jniLibs"

        commandLine(
            "wsl.exe", "bash", "-c",
            "cd '$wslRoot/lib' && " +
            "ANDROID_NDK_HOME='$wslNdk' " +
            "\$HOME/.cargo/bin/cargo ndk " +
            "-t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 " +
            "-o '$wslOut' " +
            "build -p quirq-ffi"
        )
    } else {
        // WSL2 / Linux / CI path
        workingDir(file("../../../lib"))

        doFirst {
            environment("ANDROID_NDK_HOME", android.ndkDirectory.absolutePath)
            environment("ANDROID_NDK_ROOT", android.ndkDirectory.absolutePath)
            val cargoPath = "${System.getProperty("user.home")}/.cargo/bin"
            val currentPath = System.getenv("PATH") ?: ""
            environment("PATH", "$cargoPath:$currentPath")
        }

        commandLine(
            "cargo", "ndk",
            "-t", "arm64-v8a",
            "-t", "armeabi-v7a",
            "-t", "x86_64",
            "-t", "x86",
            "-o", jniLibsDir.absolutePath,
            "build", "-p", "quirq-ffi",
        )
    }
}

tasks.named("preBuild") {
    dependsOn(cargoBuild)
}
