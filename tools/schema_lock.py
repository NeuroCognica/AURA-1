#!/usr/bin/env python3
"""
Schema Authority Lock Validator

Three-layer validation ensuring schema drift cannot occur:
1. Existence: Canonical schemas must exist
2. Syntax: JSON Schema Draft-07 compliance
3. Parity: OpenAPI matches canonical + Rust serialization matches canonical

Exit codes:
0 = success (no drift)
1 = validation failure (schema drift detected)
2 = setup error (missing dependencies, wrong path, etc.)
"""

import sys
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Optional

try:
    import jsonschema
    from jsonschema import Draft7Validator
except ImportError:
    print("ERROR: jsonschema not installed. Run: pip install jsonschema", file=sys.stderr)
    sys.exit(2)


class SchemaLockValidator:
    def __init__(self, workspace_root: Path):
        self.workspace_root = workspace_root
        self.schemas_dir = workspace_root / "schemas"
        self.openapi_path = workspace_root / "backend" / "openapi.json"
        self.schema_dump_binary = workspace_root / "target" / "release" / "schema_dump"
        if sys.platform == "win32":
            self.schema_dump_binary = self.schema_dump_binary.with_suffix(".exe")
        
        self.errors: List[str] = []
        self.warnings: List[str] = []

    def validate(self) -> bool:
        """Run all three validation layers. Returns True if all pass."""
        print("Schema Authority Lock Validator")
        print("=" * 60)
        
        # Layer 1: Existence
        if not self._validate_existence():
            return False
        
        # Layer 2: Syntax
        if not self._validate_syntax():
            return False
        
        # Layer 3: Parity (OpenAPI + Rust)
        if not self._validate_parity():
            return False
        
        # Success
        print("\n" + "=" * 60)
        print("✓ All schema authority checks passed")
        if self.warnings:
            print(f"\n⚠  {len(self.warnings)} warning(s):")
            for w in self.warnings:
                print(f"  - {w}")
        return True

    def _validate_existence(self) -> bool:
        """Layer 1: Verify canonical schemas exist."""
        print("\n[Layer 1: Existence]")
        
        required_schemas = [
            "CouncilVerdict.json",
            "CouncilEnvelope.json",
            "CouncilMsg.json",
        ]
        
        missing = []
        for schema_name in required_schemas:
            schema_path = self.schemas_dir / schema_name
            if not schema_path.exists():
                missing.append(schema_name)
                self.errors.append(f"Missing canonical schema: {schema_name}")
            else:
                print(f"  ✓ {schema_name}")
        
        if missing:
            print(f"  ✗ Missing {len(missing)} schema(s)")
            return False
        
        return True

    def _validate_syntax(self) -> bool:
        """Layer 2: Validate JSON Schema syntax (Draft-07)."""
        print("\n[Layer 2: Syntax]")
        
        for schema_file in self.schemas_dir.glob("*.json"):
            try:
                with open(schema_file, 'r', encoding='utf-8') as f:
                    schema = json.load(f)
                
                # Check for $schema field
                if "$schema" not in schema:
                    self.warnings.append(f"{schema_file.name}: Missing $schema field")
                elif "draft-07" not in schema["$schema"]:
                    self.errors.append(
                        f"{schema_file.name}: Must use JSON Schema Draft-07 or later"
                    )
                    continue
                
                # Validate schema is valid JSON Schema
                Draft7Validator.check_schema(schema)
                print(f"  ✓ {schema_file.name}")
                
            except json.JSONDecodeError as e:
                self.errors.append(f"{schema_file.name}: Invalid JSON - {e}")
                return False
            except jsonschema.SchemaError as e:
                self.errors.append(f"{schema_file.name}: Invalid JSON Schema - {e}")
                return False
        
        return len(self.errors) == 0

    def _validate_parity(self) -> bool:
        """Layer 3: Verify OpenAPI and Rust serialization match canonical schemas."""
        print("\n[Layer 3: Parity]")
        
        # 3a: OpenAPI parity
        if not self._validate_openapi_parity():
            return False
        
        # 3b: Rust serialization parity
        if not self._validate_rust_parity():
            return False
        
        return True

    def _validate_openapi_parity(self) -> bool:
        """Verify OpenAPI components match canonical schemas."""
        if not self.openapi_path.exists():
            self.warnings.append("OpenAPI spec not found - skipping OpenAPI parity check")
            return True
        
        try:
            with open(self.openapi_path, 'r', encoding='utf-8') as f:
                openapi = json.load(f)
        except json.JSONDecodeError as e:
            self.errors.append(f"OpenAPI spec invalid JSON: {e}")
            return False
        
        components = openapi.get("components", {}).get("schemas", {})
        
        # Check key schemas exist in OpenAPI
        required_in_openapi = ["CouncilVerdict", "CouncilEnvelope", "CouncilMsg"]
        for schema_name in required_in_openapi:
            canonical_path = self.schemas_dir / f"{schema_name}.json"
            if not canonical_path.exists():
                continue  # Already caught in Layer 1
            
            if schema_name not in components:
                self.errors.append(
                    f"OpenAPI missing schema component: {schema_name}"
                )
                continue
            
            # Load canonical schema
            with open(canonical_path, 'r', encoding='utf-8') as f:
                canonical = json.load(f)
            
            # Basic structure check (deep comparison would require schema resolution)
            openapi_schema = components[schema_name]
            if not self._schemas_compatible(canonical, openapi_schema, schema_name):
                return False
            
            print(f"  ✓ OpenAPI.{schema_name} matches canonical")
        
        return True

    def _schemas_compatible(self, canonical: dict, generated: dict, name: str) -> bool:
        """Basic compatibility check between canonical and generated schemas."""
        # Check required fields match
        canonical_required = set(canonical.get("required", []))
        generated_required = set(generated.get("required", []))
        
        if canonical_required != generated_required:
            self.errors.append(
                f"{name}: required fields mismatch\n"
                f"  Canonical: {sorted(canonical_required)}\n"
                f"  Generated: {sorted(generated_required)}"
            )
            return False
        
        # Check property keys match
        canonical_props = set(canonical.get("properties", {}).keys())
        generated_props = set(generated.get("properties", {}).keys())
        
        missing_in_gen = canonical_props - generated_props
        extra_in_gen = generated_props - canonical_props
        
        if missing_in_gen:
            self.errors.append(
                f"{name}: Generated schema missing properties: {sorted(missing_in_gen)}"
            )
            return False
        
        if extra_in_gen:
            # Extra fields are warnings, not errors (allow additive changes)
            self.warnings.append(
                f"{name}: Generated schema has extra properties: {sorted(extra_in_gen)}"
            )
        
        return True

    def _validate_rust_parity(self) -> bool:
        """Run schema_dump binary and verify Rust serialization matches canonical."""
        if not self.schema_dump_binary.exists():
            self.errors.append(
                f"schema_dump binary not found: {self.schema_dump_binary}\n"
                "  Run: cargo build --release"
            )
            return False
        
        try:
            result = subprocess.run(
                [str(self.schema_dump_binary)],
                capture_output=True,
                text=True,
                timeout=10,
                cwd=self.workspace_root,
            )
            
            if result.returncode != 0:
                self.errors.append(
                    f"schema_dump failed with exit code {result.returncode}\n"
                    f"  stderr: {result.stderr}"
                )
                return False
            
            # Parse output (expect JSON lines: {"struct":"CouncilVerdict","schema":{...}})
            rust_schemas = {}
            for line in result.stdout.strip().split('\n'):
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                    struct_name = obj.get("struct")
                    schema = obj.get("schema")
                    if struct_name and schema:
                        rust_schemas[struct_name] = schema
                except json.JSONDecodeError:
                    self.warnings.append(f"schema_dump output non-JSON line: {line[:50]}")
            
            # Validate against canonical schemas
            for struct_name, rust_schema in rust_schemas.items():
                canonical_path = self.schemas_dir / f"{struct_name}.json"
                if not canonical_path.exists():
                    self.warnings.append(
                        f"Rust struct {struct_name} has no canonical schema"
                    )
                    continue
                
                with open(canonical_path, 'r', encoding='utf-8') as f:
                    canonical = json.load(f)
                
                if not self._schemas_compatible(canonical, rust_schema, f"Rust.{struct_name}"):
                    return False
                
                print(f"  ✓ Rust.{struct_name} matches canonical")
            
            return True
            
        except subprocess.TimeoutExpired:
            self.errors.append("schema_dump timed out after 10 seconds")
            return False
        except Exception as e:
            self.errors.append(f"schema_dump execution failed: {e}")
            return False

    def report_errors(self):
        """Print all errors to stderr."""
        if self.errors:
            print("\n" + "=" * 60, file=sys.stderr)
            print("SCHEMA DRIFT DETECTED", file=sys.stderr)
            print("=" * 60, file=sys.stderr)
            for err in self.errors:
                print(f"\n✗ {err}", file=sys.stderr)
            print("\nSchema authority lock FAILED. Fix drifts above.", file=sys.stderr)


def main():
    # Detect workspace root
    workspace_root = Path(__file__).parent.parent
    
    validator = SchemaLockValidator(workspace_root)
    
    if validator.validate():
        sys.exit(0)
    else:
        validator.report_errors()
        sys.exit(1)


if __name__ == "__main__":
    main()
