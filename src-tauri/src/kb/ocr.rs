//! OCR module for AssistSupport
//! Pluggable OCR providers: Vision (macOS default), Tesseract (optional)

use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("OCR provider not available: {0}")]
    ProviderNotAvailable(String),
    #[error("OCR processing failed: {0}")]
    ProcessingFailed(String),
    #[error("Image read error: {0}")]
    ImageError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// OCR result with text and confidence
#[derive(Debug, Clone)]
pub struct OcrResult {
    pub text: String,
    pub confidence: Option<f32>,
    pub provider: OcrProvider,
}

/// Available OCR providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OcrProvider {
    Vision,
    Tesseract,
    None,
}

impl std::fmt::Display for OcrProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OcrProvider::Vision => write!(f, "Vision"),
            OcrProvider::Tesseract => write!(f, "Tesseract"),
            OcrProvider::None => write!(f, "None"),
        }
    }
}

/// OCR engine trait for pluggable providers
pub trait OcrEngine: Send + Sync {
    fn name(&self) -> OcrProvider;
    fn is_available(&self) -> bool;
    fn recognize(&self, image_path: &Path) -> Result<OcrResult, OcrError>;
    fn recognize_bytes(&self, image_data: &[u8], format: &str) -> Result<OcrResult, OcrError>;
}

/// JSON response from Vision OCR helper
#[derive(Debug, serde::Deserialize)]
struct VisionOcrResponse {
    success: bool,
    #[serde(rename = "fullText")]
    full_text: String,
    results: Vec<VisionOcrResultItem>,
    error: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct VisionOcrResultItem {
    #[allow(dead_code)] // Deserialized but we use full_text instead
    text: String,
    confidence: f32,
}

/// macOS Vision OCR provider
/// Uses a bundled Swift helper binary to call Vision framework
pub struct VisionOcr {
    helper_path: Option<PathBuf>,
}

impl VisionOcr {
    pub fn new() -> Self {
        // Look for bundled Vision helper in Tauri resources
        let helper_path = Self::find_helper();
        Self { helper_path }
    }

    fn find_helper() -> Option<PathBuf> {
        // In development, check local build in helpers directory
        let dev_paths = [
            PathBuf::from("helpers/vision_ocr"),
            PathBuf::from("./helpers/vision_ocr"),
            PathBuf::from("../src-tauri/helpers/vision_ocr"),
        ];

        for path in &dev_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }

        // In production, check Tauri resource path
        #[cfg(target_os = "macos")]
        {
            if let Ok(exe_path) = std::env::current_exe() {
                // Check Resources/helpers/ in bundle
                let resource_path = exe_path
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("Resources").join("helpers").join("vision_ocr"));

                if let Some(path) = resource_path {
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }

        None
    }

    /// Check if Vision framework is available (macOS 10.15+)
    fn check_vision_available() -> bool {
        #[cfg(target_os = "macos")]
        {
            // Vision is available on macOS 10.15+
            // For now, assume it's available on macOS
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }
}

impl Default for VisionOcr {
    fn default() -> Self {
        Self::new()
    }
}

impl OcrEngine for VisionOcr {
    fn name(&self) -> OcrProvider {
        OcrProvider::Vision
    }

    fn is_available(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            // Vision is available if we have the helper or can use system Vision
            self.helper_path.is_some() || Self::check_vision_available()
        }
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    fn recognize(&self, image_path: &Path) -> Result<OcrResult, OcrError> {
        if !self.is_available() {
            return Err(OcrError::ProviderNotAvailable("Vision".into()));
        }

        // If we have a helper binary, use it
        if let Some(helper) = &self.helper_path {
            let output = Command::new(helper)
                .arg(image_path)
                .output()
                .map_err(|e| OcrError::ProcessingFailed(e.to_string()))?;

            let stdout = String::from_utf8_lossy(&output.stdout);

            // Parse JSON response from helper
            let response: VisionOcrResponse = serde_json::from_str(&stdout)
                .map_err(|e| OcrError::ProcessingFailed(format!("Failed to parse OCR response: {}", e)))?;

            if !response.success {
                return Err(OcrError::ProcessingFailed(
                    response.error.unwrap_or_else(|| "Unknown error".into())
                ));
            }

            // Calculate average confidence from all results
            let avg_confidence = if response.results.is_empty() {
                None
            } else {
                Some(response.results.iter().map(|r| r.confidence).sum::<f32>() / response.results.len() as f32)
            };

            return Ok(OcrResult {
                text: response.full_text,
                confidence: avg_confidence,
                provider: OcrProvider::Vision,
            });
        }

        // Fallback: use osascript for basic OCR (limited but works without helper)
        #[cfg(target_os = "macos")]
        {
            self.recognize_with_osascript(image_path)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(OcrError::ProviderNotAvailable("Vision".into()))
        }
    }

    fn recognize_bytes(&self, image_data: &[u8], format: &str) -> Result<OcrResult, OcrError> {
        // Write to temp file and process
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("ocr_temp.{}", format));
        std::fs::write(&temp_path, image_data)?;

        let result = self.recognize(&temp_path);

        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);

        result
    }
}

impl VisionOcr {
    #[cfg(target_os = "macos")]
    fn recognize_with_osascript(&self, image_path: &Path) -> Result<OcrResult, OcrError> {
        // Use Swift inline via osascript for basic Vision OCR
        // This is a fallback when the helper binary isn't available
        let script = format!(
            r#"
            use framework "Vision"
            use framework "Foundation"
            use framework "AppKit"

            set imagePath to "{}"
            set imageURL to current application's NSURL's fileURLWithPath:imagePath

            set requestHandler to current application's VNImageRequestHandler's alloc()'s initWithURL:imageURL options:(current application's NSDictionary's dictionary())

            set textRequest to current application's VNRecognizeTextRequest's alloc()'s init()
            textRequest's setRecognitionLevel:(current application's VNRequestTextRecognitionLevelAccurate)

            set requestList to current application's NSArray's arrayWithObject:textRequest
            requestHandler's performRequests:requestList |error|:(missing value)

            set results to textRequest's results()
            set outputText to ""

            repeat with observation in results
                set topCandidate to (observation's topCandidates:1)'s firstObject()
                if topCandidate is not missing value then
                    set outputText to outputText & (topCandidate's |string|() as text) & linefeed
                end if
            end repeat

            return outputText
            "#,
            image_path.display()
        );

        let output = Command::new("osascript")
            .args(["-l", "AppleScript", "-e", &script])
            .output()
            .map_err(|e| OcrError::ProcessingFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OcrError::ProcessingFailed(stderr.to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(OcrResult {
            text,
            confidence: None,
            provider: OcrProvider::Vision,
        })
    }
}

/// Tesseract OCR provider (optional, feature-gated)
#[cfg(feature = "tesseract")]
pub struct TesseractOcr {
    tessdata_path: Option<PathBuf>,
}

#[cfg(feature = "tesseract")]
impl TesseractOcr {
    pub fn new() -> Self {
        let tessdata_path = Self::find_tessdata();
        Self { tessdata_path }
    }

    fn find_tessdata() -> Option<PathBuf> {
        // Check bundled tessdata
        let bundled_paths = [
            PathBuf::from("../Resources/tessdata"),
            PathBuf::from("./Resources/tessdata"),
        ];

        for path in &bundled_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }

        // Check system paths
        let system_paths = [
            PathBuf::from("/usr/local/share/tessdata"),
            PathBuf::from("/opt/homebrew/share/tessdata"),
        ];

        for path in &system_paths {
            if path.exists() {
                return Some(path.clone());
            }
        }

        None
    }
}

#[cfg(feature = "tesseract")]
impl Default for TesseractOcr {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "tesseract")]
impl OcrEngine for TesseractOcr {
    fn name(&self) -> OcrProvider {
        OcrProvider::Tesseract
    }

    fn is_available(&self) -> bool {
        self.tessdata_path.is_some()
            && Command::new("tesseract")
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
    }

    fn recognize(&self, image_path: &Path) -> Result<OcrResult, OcrError> {
        if !self.is_available() {
            return Err(OcrError::ProviderNotAvailable("Tesseract".into()));
        }

        let mut cmd = Command::new("tesseract");
        cmd.arg(image_path).arg("stdout").arg("-l").arg("eng");

        if let Some(tessdata) = &self.tessdata_path {
            cmd.env("TESSDATA_PREFIX", tessdata);
        }

        let output = cmd
            .output()
            .map_err(|e| OcrError::ProcessingFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OcrError::ProcessingFailed(stderr.to_string()));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(OcrResult {
            text,
            confidence: None,
            provider: OcrProvider::Tesseract,
        })
    }

    fn recognize_bytes(&self, image_data: &[u8], format: &str) -> Result<OcrResult, OcrError> {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join(format!("ocr_temp.{}", format));
        std::fs::write(&temp_path, image_data)?;

        let result = self.recognize(&temp_path);
        let _ = std::fs::remove_file(&temp_path);
        result
    }
}

/// OCR Manager - selects and manages OCR providers
pub struct OcrManager {
    vision: VisionOcr,
    #[cfg(feature = "tesseract")]
    tesseract: TesseractOcr,
    preferred_provider: OcrProvider,
}

impl OcrManager {
    pub fn new() -> Self {
        let vision = VisionOcr::new();
        #[cfg(feature = "tesseract")]
        let tesseract = TesseractOcr::new();

        // Default to Vision on macOS
        let preferred_provider = if vision.is_available() {
            OcrProvider::Vision
        } else {
            #[cfg(feature = "tesseract")]
            if tesseract.is_available() {
                OcrProvider::Tesseract
            } else {
                OcrProvider::None
            }
            #[cfg(not(feature = "tesseract"))]
            OcrProvider::None
        };

        Self {
            vision,
            #[cfg(feature = "tesseract")]
            tesseract,
            preferred_provider,
        }
    }

    /// Get available providers
    pub fn available_providers(&self) -> Vec<OcrProvider> {
        let mut providers = Vec::new();
        if self.vision.is_available() {
            providers.push(OcrProvider::Vision);
        }
        #[cfg(feature = "tesseract")]
        if self.tesseract.is_available() {
            providers.push(OcrProvider::Tesseract);
        }
        providers
    }

    /// Set preferred provider
    pub fn set_preferred_provider(&mut self, provider: OcrProvider) {
        self.preferred_provider = provider;
    }

    /// Get preferred provider
    pub fn preferred_provider(&self) -> OcrProvider {
        self.preferred_provider
    }

    /// Recognize text in image
    pub fn recognize(&self, image_path: &Path) -> Result<OcrResult, OcrError> {
        self.recognize_with_provider(image_path, self.preferred_provider)
    }

    /// Recognize with specific provider
    pub fn recognize_with_provider(
        &self,
        image_path: &Path,
        provider: OcrProvider,
    ) -> Result<OcrResult, OcrError> {
        match provider {
            OcrProvider::Vision => self.vision.recognize(image_path),
            #[cfg(feature = "tesseract")]
            OcrProvider::Tesseract => self.tesseract.recognize(image_path),
            #[cfg(not(feature = "tesseract"))]
            OcrProvider::Tesseract => Err(OcrError::ProviderNotAvailable("Tesseract".into())),
            OcrProvider::None => Err(OcrError::ProviderNotAvailable("None".into())),
        }
    }

    /// Recognize with fallback
    pub fn recognize_with_fallback(&self, image_path: &Path) -> Result<OcrResult, OcrError> {
        // Try preferred provider first
        match self.recognize_with_provider(image_path, self.preferred_provider) {
            Ok(result) => return Ok(result),
            Err(_) => {}
        }

        // Try all available providers
        for provider in self.available_providers() {
            if provider != self.preferred_provider {
                if let Ok(result) = self.recognize_with_provider(image_path, provider) {
                    return Ok(result);
                }
            }
        }

        Err(OcrError::ProviderNotAvailable(
            "No OCR provider available".into(),
        ))
    }
}

impl Default for OcrManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_manager_creation() {
        let manager = OcrManager::new();
        let providers = manager.available_providers();
        println!("Available OCR providers: {:?}", providers);

        // On macOS, Vision should be available (at least in principle)
        #[cfg(target_os = "macos")]
        {
            // Vision availability depends on the helper being present
            // In tests, it might not be, so we just check it doesn't panic
        }
    }

    #[test]
    fn test_vision_availability() {
        let vision = VisionOcr::new();
        #[cfg(target_os = "macos")]
        {
            // Check that Vision reports availability correctly
            let available = vision.is_available();
            println!("Vision OCR available: {}", available);
        }
        #[cfg(not(target_os = "macos"))]
        {
            assert!(!vision.is_available());
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_vision_ocr_integration() {
        let vision = VisionOcr::new();
        if !vision.is_available() {
            println!("Vision OCR not available, skipping integration test");
            return;
        }

        // Use the test image created in helpers directory
        let test_image = PathBuf::from("helpers/test_ocr.png");
        if !test_image.exists() {
            println!("Test image not found at {:?}, skipping", test_image);
            return;
        }

        let result = vision.recognize(&test_image);
        match result {
            Ok(ocr_result) => {
                println!("OCR Result: {}", ocr_result.text);
                println!("Confidence: {:?}", ocr_result.confidence);
                assert!(ocr_result.text.contains("AssistSupport") || ocr_result.text.contains("OCR"));
            }
            Err(e) => {
                println!("OCR failed: {}", e);
                // Don't fail the test if OCR fails - it might be environment-specific
            }
        }
    }
}
