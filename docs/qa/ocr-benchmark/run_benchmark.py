#!/usr/bin/env python3
"""
OCR Benchmark Runner for AssistSupport
Evaluates Vision OCR accuracy using Word Error Rate (WER)
"""

import json
import os
import subprocess
import sys
from pathlib import Path
from jiwer import wer, cer

def run_ocr(image_path: str, vision_ocr_path: str) -> tuple[str, float]:
    """Run Vision OCR on an image and return (text, confidence)"""
    try:
        result = subprocess.run(
            [vision_ocr_path, image_path],
            capture_output=True,
            text=True,
            timeout=30
        )
        if result.returncode != 0:
            return "", 0.0

        data = json.loads(result.stdout)
        if data.get("success"):
            return data.get("fullText", ""), sum(r.get("confidence", 0) for r in data.get("results", [])) / max(len(data.get("results", [])), 1)
        return "", 0.0
    except Exception as e:
        print(f"  Error processing {image_path}: {e}")
        return "", 0.0

def main():
    # Paths
    script_dir = Path(__file__).parent
    images_dir = script_dir / "images"
    gt_dir = script_dir / "ground-truth"
    results_file = script_dir / "benchmark_results.json"

    # Find Vision OCR helper
    vision_ocr_candidates = [
        script_dir.parent.parent.parent / "src-tauri" / "helpers" / "vision_ocr",
        Path("src-tauri/helpers/vision_ocr"),
    ]

    vision_ocr_path = None
    for candidate in vision_ocr_candidates:
        if candidate.exists():
            vision_ocr_path = str(candidate)
            break

    if not vision_ocr_path:
        print("ERROR: Vision OCR helper not found")
        sys.exit(1)

    print(f"Using Vision OCR: {vision_ocr_path}")
    print(f"Images directory: {images_dir}")
    print(f"Ground truth directory: {gt_dir}")
    print()

    # Collect results
    results = []
    total_wer = 0.0
    total_cer = 0.0
    processed = 0

    image_files = sorted(images_dir.glob("*.png"))
    print(f"Processing {len(image_files)} images...\n")

    for image_path in image_files:
        gt_path = gt_dir / (image_path.stem + ".txt")

        if not gt_path.exists():
            print(f"  SKIP: No ground truth for {image_path.name}")
            continue

        # Read ground truth
        ground_truth = gt_path.read_text().strip()

        # Run OCR
        ocr_text, confidence = run_ocr(str(image_path), vision_ocr_path)
        ocr_text = ocr_text.strip()

        # Calculate WER and CER
        if ground_truth and ocr_text:
            img_wer = wer(ground_truth, ocr_text)
            img_cer = cer(ground_truth, ocr_text)
        else:
            img_wer = 1.0  # 100% error if empty
            img_cer = 1.0

        # Determine image type from filename
        is_image_only = any(
            image_path.name.startswith(prefix) and int(image_path.stem.split("_")[1]) <= threshold
            for prefix, threshold in [
                ("error_", 8),
                ("instruction_", 3),
                ("technical_", 3),
                ("network_", 2),
                ("multiline_", 4),
            ]
        )

        result = {
            "image": image_path.name,
            "ground_truth": ground_truth[:50] + "..." if len(ground_truth) > 50 else ground_truth,
            "ocr_output": ocr_text[:50] + "..." if len(ocr_text) > 50 else ocr_text,
            "wer": round(img_wer, 4),
            "cer": round(img_cer, 4),
            "confidence": round(confidence, 4),
            "is_image_only": is_image_only,
            "exact_match": ground_truth == ocr_text,
        }
        results.append(result)

        total_wer += img_wer
        total_cer += img_cer
        processed += 1

        # Progress indicator
        status = "EXACT" if result["exact_match"] else f"WER={img_wer:.2%}"
        type_indicator = "[IMG]" if is_image_only else "[TXT]"
        print(f"  {type_indicator} {image_path.name}: {status}")

    # Calculate summary statistics
    avg_wer = total_wer / max(processed, 1)
    avg_cer = total_cer / max(processed, 1)
    exact_matches = sum(1 for r in results if r["exact_match"])

    # Image-only vs text statistics
    image_only_results = [r for r in results if r["is_image_only"]]
    text_results = [r for r in results if not r["is_image_only"]]

    image_only_wer = sum(r["wer"] for r in image_only_results) / max(len(image_only_results), 1)
    text_wer = sum(r["wer"] for r in text_results) / max(len(text_results), 1)

    summary = {
        "total_images": processed,
        "exact_matches": exact_matches,
        "exact_match_rate": round(exact_matches / max(processed, 1), 4),
        "average_wer": round(avg_wer, 4),
        "average_cer": round(avg_cer, 4),
        "image_only_count": len(image_only_results),
        "image_only_wer": round(image_only_wer, 4),
        "text_based_count": len(text_results),
        "text_based_wer": round(text_wer, 4),
    }

    # Save results
    output = {
        "summary": summary,
        "results": results,
    }

    results_file.write_text(json.dumps(output, indent=2))

    # Print summary
    print("\n" + "=" * 60)
    print("OCR BENCHMARK RESULTS")
    print("=" * 60)
    print(f"Total Images:        {summary['total_images']}")
    print(f"Exact Matches:       {summary['exact_matches']} ({summary['exact_match_rate']:.1%})")
    print(f"Average WER:         {summary['average_wer']:.2%}")
    print(f"Average CER:         {summary['average_cer']:.2%}")
    print()
    print("By Image Type:")
    print(f"  Image-only ({summary['image_only_count']} images): WER = {summary['image_only_wer']:.2%}")
    print(f"  Text-based ({summary['text_based_count']} images): WER = {summary['text_based_wer']:.2%}")
    print()
    print(f"Results saved to: {results_file}")

    # Gate check (per implementation plan)
    WER_GATE = 0.05  # 5% WER threshold
    if avg_wer <= WER_GATE:
        print(f"\n✅ PASSED: WER ({avg_wer:.2%}) <= {WER_GATE:.0%} threshold")
        sys.exit(0)
    else:
        print(f"\n❌ FAILED: WER ({avg_wer:.2%}) > {WER_GATE:.0%} threshold")
        sys.exit(1)

if __name__ == "__main__":
    main()
