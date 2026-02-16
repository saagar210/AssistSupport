#!/usr/bin/env swift
import AppKit

let width = 400
let height = 100
let text = "Hello AssistSupport OCR Test"

let image = NSImage(size: NSSize(width: width, height: height))
image.lockFocus()

NSColor.white.setFill()
NSRect(x: 0, y: 0, width: width, height: height).fill()

let font = NSFont.systemFont(ofSize: 24)
let attributes: [NSAttributedString.Key: Any] = [
    .font: font,
    .foregroundColor: NSColor.black
]

let textSize = text.size(withAttributes: attributes)
let x = (CGFloat(width) - textSize.width) / 2
let y = (CGFloat(height) - textSize.height) / 2

text.draw(at: NSPoint(x: x, y: y), withAttributes: attributes)
image.unlockFocus()

if let tiffData = image.tiffRepresentation,
   let bitmap = NSBitmapImageRep(data: tiffData),
   let pngData = bitmap.representation(using: .png, properties: [:]) {
    try! pngData.write(to: URL(fileURLWithPath: "test_ocr.png"))
    print("Created test_ocr.png")
}
