#!/usr/bin/env swift
// Generates OCR benchmark images for WER evaluation
// Creates 50+ images with various text content types

import AppKit
import Foundation

// IT Support related phrases for realistic benchmark
let errorMessages = [
    "Error: Connection timed out after 30 seconds",
    "WARNING: Disk space low on /dev/sda1",
    "Failed to authenticate user: Invalid credentials",
    "Service 'nginx' restarted successfully",
    "CRITICAL: Memory usage exceeded 95%",
    "Permission denied: Access to /etc/passwd",
    "Network interface eth0 is DOWN",
    "SSL certificate expires in 7 days",
    "Database backup completed: 2.5GB",
    "Firewall rule added: Allow TCP 443",
    "Process apache2 killed (PID 1234)",
    "USB device disconnected unexpectedly",
    "SMART warning: Bad sectors detected",
    "Login attempt from unknown IP: 192.168.1.100",
    "System reboot required for kernel update",
    "DNS resolution failed for api.example.com",
    "Cache cleared: 1.2GB freed",
    "User account locked after 3 failed attempts",
    "Antivirus scan completed: No threats found",
    "VPN connection established to 10.0.0.1"
]

let instructions = [
    "Step 1: Open Terminal and run 'sudo apt update'",
    "Step 2: Install required packages with 'apt install nginx'",
    "To reset password: Click Settings > Security > Reset",
    "Configure firewall: ufw allow 22/tcp",
    "Check disk usage: df -h /home",
    "Restart service: systemctl restart apache2",
    "View logs: tail -f /var/log/syslog",
    "Backup database: pg_dump mydb > backup.sql",
    "Update software: brew upgrade --all",
    "Check network: ping -c 4 google.com"
]

let technicalText = [
    "CPU: Intel Core i7-12700K @ 3.6GHz",
    "RAM: 32GB DDR5-4800 (2x16GB)",
    "Storage: 1TB NVMe SSD (Samsung 980 Pro)",
    "GPU: NVIDIA RTX 3080 12GB VRAM",
    "OS: macOS Sequoia 15.2 (24C101)",
    "Kernel: Darwin 24.2.0 arm64",
    "Python 3.12.0 | pip 23.3.1",
    "Node.js v22.5.1 | npm 10.8.2",
    "Docker Desktop 4.35.0 (173168)",
    "Homebrew 4.4.0 | Git 2.43.0"
]

let ipAddresses = [
    "192.168.1.1 gateway",
    "10.0.0.50 fileserver",
    "172.16.0.100 database",
    "fe80::1 link-local",
    "2001:db8::1 ipv6-host"
]

func createImage(width: Int, height: Int, text: String, fontSize: CGFloat, isImageOnly: Bool = false) -> NSImage {
    let image = NSImage(size: NSSize(width: width, height: height))
    image.lockFocus()

    // Background - white for text, light gray for "scanned" image-only
    if isImageOnly {
        NSColor(white: 0.95, alpha: 1.0).setFill()
    } else {
        NSColor.white.setFill()
    }
    NSRect(x: 0, y: 0, width: width, height: height).fill()

    // Add some noise/texture for image-only pages (simulate scan)
    if isImageOnly {
        for _ in 0..<50 {
            let x = CGFloat.random(in: 0...CGFloat(width))
            let y = CGFloat.random(in: 0...CGFloat(height))
            NSColor(white: CGFloat.random(in: 0.8...0.98), alpha: 1.0).setFill()
            NSRect(x: x, y: y, width: 2, height: 2).fill()
        }
    }

    // Draw text
    let font = NSFont.monospacedSystemFont(ofSize: fontSize, weight: .regular)
    let paragraphStyle = NSMutableParagraphStyle()
    paragraphStyle.lineSpacing = 4

    let attributes: [NSAttributedString.Key: Any] = [
        .font: font,
        .foregroundColor: isImageOnly ? NSColor(white: 0.15, alpha: 1.0) : NSColor.black,
        .paragraphStyle: paragraphStyle
    ]

    let textRect = NSRect(x: 20, y: 20, width: width - 40, height: height - 40)
    text.draw(in: textRect, withAttributes: attributes)

    image.unlockFocus()
    return image
}

func saveImage(_ image: NSImage, to path: String) {
    guard let tiffData = image.tiffRepresentation,
          let bitmap = NSBitmapImageRep(data: tiffData),
          let pngData = bitmap.representation(using: .png, properties: [:]) else {
        print("Failed to save image: \(path)")
        return
    }
    do {
        try pngData.write(to: URL(fileURLWithPath: path))
    } catch {
        print("Error writing \(path): \(error)")
    }
}

// Generate benchmark images
let basePath = FileManager.default.currentDirectoryPath + "/images"
let gtPath = FileManager.default.currentDirectoryPath + "/ground-truth"

var imageCount = 0
var imageOnlyCount = 0

// Error messages (20 images, 8 image-only)
for (i, msg) in errorMessages.enumerated() {
    let isImageOnly = i < 8
    let image = createImage(width: 600, height: 100, text: msg, fontSize: 16, isImageOnly: isImageOnly)
    let filename = String(format: "error_%02d.png", i + 1)
    saveImage(image, to: basePath + "/" + filename)
    try! msg.write(toFile: gtPath + "/" + filename.replacingOccurrences(of: ".png", with: ".txt"), atomically: true, encoding: .utf8)
    imageCount += 1
    if isImageOnly { imageOnlyCount += 1 }
}

// Instructions (10 images, 3 image-only)
for (i, instr) in instructions.enumerated() {
    let isImageOnly = i < 3
    let image = createImage(width: 600, height: 80, text: instr, fontSize: 14, isImageOnly: isImageOnly)
    let filename = String(format: "instruction_%02d.png", i + 1)
    saveImage(image, to: basePath + "/" + filename)
    try! instr.write(toFile: gtPath + "/" + filename.replacingOccurrences(of: ".png", with: ".txt"), atomically: true, encoding: .utf8)
    imageCount += 1
    if isImageOnly { imageOnlyCount += 1 }
}

// Technical specs (10 images, 3 image-only)
for (i, spec) in technicalText.enumerated() {
    let isImageOnly = i < 3
    let image = createImage(width: 500, height: 60, text: spec, fontSize: 14, isImageOnly: isImageOnly)
    let filename = String(format: "technical_%02d.png", i + 1)
    saveImage(image, to: basePath + "/" + filename)
    try! spec.write(toFile: gtPath + "/" + filename.replacingOccurrences(of: ".png", with: ".txt"), atomically: true, encoding: .utf8)
    imageCount += 1
    if isImageOnly { imageOnlyCount += 1 }
}

// IP addresses (5 images, 2 image-only)
for (i, ip) in ipAddresses.enumerated() {
    let isImageOnly = i < 2
    let image = createImage(width: 400, height: 50, text: ip, fontSize: 14, isImageOnly: isImageOnly)
    let filename = String(format: "network_%02d.png", i + 1)
    saveImage(image, to: basePath + "/" + filename)
    try! ip.write(toFile: gtPath + "/" + filename.replacingOccurrences(of: ".png", with: ".txt"), atomically: true, encoding: .utf8)
    imageCount += 1
    if isImageOnly { imageOnlyCount += 1 }
}

// Multi-line content (10 images, 4 image-only)
let multilineContent = [
    "System Status Report\n-------------------\nCPU: 45% | RAM: 8.2GB/16GB\nDisk: 234GB free of 500GB",
    "User Session Log\n================\nLogin: 09:15 AM\nIP: 192.168.1.50\nDuration: 2h 34m",
    "Error Stack Trace:\nTypeError: Cannot read property\n  at Object.<anonymous> (app.js:42)\n  at Module._compile (loader.js:999)",
    "Package Updates Available:\n- openssl: 3.0.12 -> 3.0.13\n- curl: 8.4.0 -> 8.5.0\n- git: 2.42 -> 2.43",
    "Network Statistics:\nPackets Sent: 1,234,567\nPackets Received: 2,345,678\nErrors: 0 | Dropped: 12",
    "Firewall Rules:\n[ALLOW] TCP 22 (SSH)\n[ALLOW] TCP 80,443 (HTTP/S)\n[DENY] UDP 53 (DNS)",
    "Backup Summary:\n- /home: 45.2 GB\n- /var/www: 12.8 GB\n- /etc: 128 MB\nTotal: 58.1 GB",
    "Process List (Top 5):\n1. chrome (12% CPU, 2.1GB)\n2. code (8% CPU, 1.5GB)\n3. node (5% CPU, 800MB)",
    "SSL Certificate Info:\nDomain: example.com\nIssuer: Let's Encrypt\nExpires: 2026-03-15",
    "Docker Containers:\n[running] nginx:latest\n[running] postgres:15\n[stopped] redis:7"
]

for (i, content) in multilineContent.enumerated() {
    let isImageOnly = i < 4
    let image = createImage(width: 500, height: 200, text: content, fontSize: 13, isImageOnly: isImageOnly)
    let filename = String(format: "multiline_%02d.png", i + 1)
    saveImage(image, to: basePath + "/" + filename)
    try! content.write(toFile: gtPath + "/" + filename.replacingOccurrences(of: ".png", with: ".txt"), atomically: true, encoding: .utf8)
    imageCount += 1
    if isImageOnly { imageOnlyCount += 1 }
}

let imageOnlyPercent = Double(imageOnlyCount) / Double(imageCount) * 100

print("OCR Benchmark Dataset Generated")
print("================================")
print("Total images: \(imageCount)")
print("Image-only: \(imageOnlyCount) (\(String(format: "%.1f", imageOnlyPercent))%)")
print("Text-based: \(imageCount - imageOnlyCount)")
print("Ground truth files: \(imageCount)")
