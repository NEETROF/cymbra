import java.io.FileInputStream
import java.util.Properties

plugins {
    id("com.android.application")
    id("kotlin-android")
    // The Flutter Gradle Plugin must be applied after the Android and Kotlin Gradle plugins.
    id("dev.flutter.flutter-gradle-plugin")
}

// Release signing (Play App Signing upload key). key.properties is gitignored and
// lives next to this module's parent (android/); absent on CI/other devs → the
// release build falls back to debug signing so `flutter run --release` still works.
// Template: android/app/key.properties.example.
val keystoreProperties = Properties()
val keystorePropertiesFile = rootProject.file("key.properties")
val hasReleaseSigning = keystorePropertiesFile.exists()
if (hasReleaseSigning) {
    keystoreProperties.load(FileInputStream(keystorePropertiesFile))
}

android {
    namespace = "org.cymbra.music"
    compileSdk = flutter.compileSdkVersion
    ndkVersion = flutter.ndkVersion

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = JavaVersion.VERSION_17.toString()
    }

    defaultConfig {
        applicationId = "com.cymbra.music"
        // AMidi (midir Android backend) requires API 29+.
        minSdk = 29
        targetSdk = flutter.targetSdkVersion
        versionCode = flutter.versionCode
        versionName = flutter.versionName
    }

    signingConfigs {
        // Shared debug-only keystore committed to the repo (app/debug.keystore) so
        // every developer and CI sign debug builds with the SAME certificate — one
        // SHA-1 to register for Google sign-in. NOT for release: add a real keystore
        // / Play App Signing and register its own SHA-1 separately.
        getByName("debug") {
            storeFile = file("debug.keystore")
            storePassword = "android"
            keyAlias = "androiddebugkey"
            keyPassword = "android"
        }
        // Real release key (Play App Signing upload key). Only registered when
        // key.properties is present so the debug fallback below keeps CI green.
        if (hasReleaseSigning) {
            create("release") {
                // storeFile is resolved relative to this module (android/app/).
                storeFile = file(keystoreProperties["storeFile"] as String)
                storePassword = keystoreProperties["storePassword"] as String
                keyAlias = keystoreProperties["keyAlias"] as String
                keyPassword = keystoreProperties["keyPassword"] as String
            }
        }
    }

    buildTypes {
        release {
            // Sign with the release upload key when configured; otherwise fall back
            // to the shared debug key so `flutter run --release` / CI still build.
            signingConfig = if (hasReleaseSigning) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
        }
    }
}

flutter {
    source = "../.."
}
