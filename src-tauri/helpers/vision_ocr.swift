#!/usr/bin/env swift
// Vision OCR Helper for AssistSupport
// Uses macOS Vision framework for text recognition

import Foundation
import Vision
import AppKit

struct OcrResult: Codable {
    let text: String
    let confidence: Float
    let boundingBox: BoundingBox?
}

struct BoundingBox: Codable {
    let x: Double
    let y: Double
    let width: Double
    let height: Double
}

struct OcrOutput: Codable {
    let success: Bool
    let results: [OcrResult]
    let fullText: String
    let error: String?
}

func performOCR(imagePath: String) -> OcrOutput {
    guard let image = NSImage(contentsOfFile: imagePath) else {
        return OcrOutput(success: false, results: [], fullText: "", error: "Failed to load image: \(imagePath)")
    }

    guard let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
        return OcrOutput(success: false, results: [], fullText: "", error: "Failed to convert image to CGImage")
    }

    var ocrResults: [OcrResult] = []
    var fullTextParts: [String] = []
    var ocrError: String? = nil

    let request = VNRecognizeTextRequest { request, error in
        if let error = error {
            ocrError = error.localizedDescription
            return
        }

        guard let observations = request.results as? [VNRecognizedTextObservation] else {
            ocrError = "No text observations found"
            return
        }

        for observation in observations {
            guard let topCandidate = observation.topCandidates(1).first else { continue }

            let boundingBox = BoundingBox(
                x: observation.boundingBox.origin.x,
                y: observation.boundingBox.origin.y,
                width: observation.boundingBox.size.width,
                height: observation.boundingBox.size.height
            )

            let result = OcrResult(
                text: topCandidate.string,
                confidence: topCandidate.confidence,
                boundingBox: boundingBox
            )

            ocrResults.append(result)
            fullTextParts.append(topCandidate.string)
        }
    }

    // Configure for best accuracy
    request.recognitionLevel = .accurate
    request.usesLanguageCorrection = true
    request.recognitionLanguages = ["en-US"]

    let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])

    do {
        try handler.perform([request])
    } catch {
        return OcrOutput(success: false, results: [], fullText: "", error: "Vision request failed: \(error.localizedDescription)")
    }

    if let error = ocrError {
        return OcrOutput(success: false, results: ocrResults, fullText: fullTextParts.joined(separator: "\n"), error: error)
    }

    return OcrOutput(success: true, results: ocrResults, fullText: fullTextParts.joined(separator: "\n"), error: nil)
}

// Main execution
guard CommandLine.arguments.count > 1 else {
    let output = OcrOutput(success: false, results: [], fullText: "", error: "Usage: vision_ocr <image_path>")
    let encoder = JSONEncoder()
    encoder.outputFormatting = .prettyPrinted
    if let json = try? encoder.encode(output), let jsonString = String(data: json, encoding: .utf8) {
        print(jsonString)
    }
    exit(1)
}

let imagePath = CommandLine.arguments[1]
let result = performOCR(imagePath: imagePath)

let encoder = JSONEncoder()
encoder.outputFormatting = .prettyPrinted
if let json = try? encoder.encode(result), let jsonString = String(data: json, encoding: .utf8) {
    print(jsonString)
    exit(result.success ? 0 : 1)
} else {
    print("{\"success\": false, \"error\": \"Failed to encode JSON output\"}")
    exit(1)
}
