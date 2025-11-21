#!/bin/bash

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
VERSION=""
BUILD_TYPES=()
BASE_URL="https://github.com/7a72/meta-magic_mount/releases/download"
UPDATE_JSON_URL="https://raw.githubusercontent.com/7a72/meta-magic_mount/public/update.json"
CHANGELOG_URL="https://raw.githubusercontent.com/7a72/meta-magic_mount/public/changelog.md"

# Show usage
usage() {
    echo "Usage: $0 [--release] [--debug]"
    echo "  --release            Build release version"
    echo "  --debug              Build debug version"
    echo "  (no option)          Build both release and debug versions"
    echo ""
    echo "Version number will be automatically obtained from git tag"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPES+=("release")
            shift
            ;;
        --debug)
            BUILD_TYPES+=("debug")
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo -e "${RED}Error: Unknown parameter $1${NC}"
            usage
            ;;
    esac
done

# If no build type specified, build both
if [ ${#BUILD_TYPES[@]} -eq 0 ]; then
    BUILD_TYPES=("release" "debug")
    echo -e "${YELLOW}No build type specified, building both release and debug versions${NC}"
fi

# Check if in git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

# Check if zig is installed
if ! command -v zig &> /dev/null; then
    echo -e "${RED}Error: zig is not installed${NC}"
    exit 1
fi

# Get git tag as version number
VERSION=$(git describe --tags --exact-match 2>/dev/null)
if [ -z "$VERSION" ]; then
    # If current commit has no tag, try to get the nearest tag
    VERSION=$(git describe --tags --abbrev=0 2>/dev/null)
    if [ -z "$VERSION" ]; then
        echo -e "${RED}Error: Unable to get git tag, please create a tag first${NC}"
        echo -e "${YELLOW}Hint: Use 'git tag v1.0.0' to create a tag${NC}"
        exit 1
    else
        echo -e "${YELLOW}Warning: Current commit has no tag, using nearest tag: $VERSION${NC}"
    fi
fi

# Get git short commit or branch name
GIT_SUFFIX=""
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null)
if [ -n "$GIT_COMMIT" ]; then
    GIT_SUFFIX="$GIT_COMMIT"
else
    # If unable to get commit, use branch name
    GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
    if [ -n "$GIT_BRANCH" ]; then
        # Clean branch name, remove special characters
        GIT_SUFFIX=$(echo "$GIT_BRANCH" | sed 's/[^a-zA-Z0-9_-]/_/g')
        echo -e "${YELLOW}Warning: Unable to get commit, using branch name: $GIT_BRANCH${NC}"
    else
        echo -e "${YELLOW}Warning: Unable to get commit and branch information${NC}"
    fi
fi

# Function to generate changelog from git commits
generate_changelog() {
    local CHANGELOG_FILE="$1"
    local PREV_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null)
    
    echo -e "${YELLOW}Generating changelog...${NC}"
    
    # Create changelog header
    cat > "$CHANGELOG_FILE" << EOF
# Changelog

## Version $VERSION ($(date +%Y-%m-%d))

### Changes

EOF

    # Add commits since last tag
    if [ -n "$PREV_TAG" ]; then
        echo "Changes since $PREV_TAG:" >> "$CHANGELOG_FILE"
        echo "" >> "$CHANGELOG_FILE"
        git log --pretty=format:"- %s (%h)" "$PREV_TAG"..HEAD >> "$CHANGELOG_FILE"
    else
        echo "Initial release" >> "$CHANGELOG_FILE"
        echo "" >> "$CHANGELOG_FILE"
        git log --pretty=format:"- %s (%h)" >> "$CHANGELOG_FILE"
    fi
    
    echo "" >> "$CHANGELOG_FILE"
    echo "" >> "$CHANGELOG_FILE"
    echo "---" >> "$CHANGELOG_FILE"
    echo "Full Changelog: https://github.com/7a72/meta-magic_mount/compare/${PREV_TAG}...${VERSION}" >> "$CHANGELOG_FILE"
    
    echo -e "${GREEN}Changelog generated: $CHANGELOG_FILE${NC}"
}

# Function to generate main update JSON (for release version)
generate_main_update_json() {
    local OUTPUT_NAME=$1
    local VERSION_CODE=$2
    local JSON_FILE="build/update.json"
    
    echo -e "${YELLOW}Generating main update JSON...${NC}"
    
    cat > "$JSON_FILE" << EOF
{
    "versionCode": $VERSION_CODE,
    "version": "$VERSION",
    "zipUrl": "$BASE_URL/$VERSION/$OUTPUT_NAME",
    "changelog": "$CHANGELOG_URL"
}
EOF
    
    echo -e "${GREEN}Main update JSON generated: $JSON_FILE${NC}"
}

# Function to build Rust project for all targets
build_binaries() {
    local BUILD_TYPE=$1
    
    echo -e "\n${YELLOW}Building binaries for all targets...${NC}"
    
    cd src || return 1
    
    # Clean previous builds
    make clean
    
    make $BUILD_TYPE VERSION="$VERSION"
    
    cd ..
    
    echo -e "${GREEN}All binaries built successfully${NC}"
    return 0
}

# Function to build for a specific type
build_for_type() {
    local BUILD_TYPE=$1
    
    echo -e "\n${GREEN}========================================${NC}"
    echo -e "${GREEN}Building $BUILD_TYPE version${NC}"
    echo -e "${GREEN}Version: $VERSION${NC}"
    echo -e "${GREEN}Build Type: $BUILD_TYPE${NC}"
    if [ -n "$GIT_COMMIT" ]; then
        echo -e "${GREEN}Git Commit: $GIT_COMMIT${NC}"
    elif [ -n "$GIT_SUFFIX" ]; then
        echo -e "${GREEN}Git Branch: $GIT_SUFFIX${NC}"
    fi
    echo -e "${GREEN}========================================${NC}"

    # Create build directory structure
    BUILD_DIR="build/${BUILD_TYPE}_temp"
    if [ -d "$BUILD_DIR" ]; then
        rm -rf "$BUILD_DIR"
    fi
    mkdir -p "$BUILD_DIR"

    # Step 1: Copy template to build directory
    echo -e "\n${YELLOW}[1/7] Copying template to build directory...${NC}"
    if [ ! -d "template" ]; then
        echo -e "${RED}Error: template directory does not exist${NC}"
        return 1
    fi

    cp -r template/* "$BUILD_DIR/"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to copy template${NC}"
        return 1
    fi
    echo -e "${GREEN}Template copied${NC}"

    # Step 2: Build Rust binaries
    echo -e "\n${YELLOW}[2/7] Building binaries...${NC}"
    build_binaries "$BUILD_TYPE"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Build failed${NC}"
        return 1
    fi
    echo -e "${GREEN}Build completed${NC}"

    # Step 3: Modify log level in metamount.sh (in build directory)
    echo -e "\n${YELLOW}[3/7] Modifying metamount.sh configuration...${NC}"
    if [ ! -f "$BUILD_DIR/metamount.sh" ]; then
        echo -e "${RED}Error: $BUILD_DIR/metamount.sh does not exist${NC}"
        return 1
    fi

    # Step 4: Modify module.prop (in build directory)
    echo -e "\n${YELLOW}[4/7] Modifying module.prop...${NC}"
    if [ ! -f "$BUILD_DIR/module.prop" ]; then
        echo -e "${RED}Error: $BUILD_DIR/module.prop does not exist${NC}"
        return 1
    fi

    # Generate versionCode
    VERSION_CODE=$(git rev-list --count HEAD)

    # Modify version, versionCode, and updateJson
    sed -i "s|^version=.*|version=$VERSION|" "$BUILD_DIR/module.prop"
    sed -i "s|^versionCode=.*|versionCode=$VERSION_CODE|" "$BUILD_DIR/module.prop"
    sed -i "s|^updateJson=.*|updateJson=$UPDATE_JSON_URL|" "$BUILD_DIR/module.prop"
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to modify module.prop${NC}"
        return 1
    fi
    echo -e "${GREEN}module.prop configuration completed${NC}"
    echo -e "${GREEN}  version=$VERSION${NC}"
    echo -e "${GREEN}  versionCode=$VERSION_CODE${NC}"
    echo -e "${GREEN}  updateJson=$UPDATE_JSON_URL${NC}"

    # Step 5: Copy bin directory to build directory
    echo -e "\n${YELLOW}[5/7] Copying bin directory...${NC}"
    if [ ! -d "src/bin" ]; then
        echo -e "${RED}Error: src/bin directory does not exist${NC}"
        return 1
    fi

    cp -r src/bin "$BUILD_DIR/"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to copy bin directory${NC}"
        return 1
    fi
    echo -e "${GREEN}bin directory copied${NC}"

    # Step 6: Package as zip
    echo -e "\n${YELLOW}[6/7] Packaging module...${NC}"

    # Build filename
    OUTPUT_NAME="meta-magic_mount-${VERSION}-${VERSION_CODE}-${GIT_SUFFIX}-${BUILD_TYPE}.zip"

    # Enter build directory and package
    cd "$BUILD_DIR" || return 1
    zip -r "../../build/$OUTPUT_NAME" ./* -x "*.git*"
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Packaging failed${NC}"
        cd ../..
        return 1
    fi
    cd ../..

    # Step 7: Generate update JSON for this build
    if [ "$BUILD_TYPE" = "release" ]; then
        generate_main_update_json "$OUTPUT_NAME" "$VERSION_CODE"
    fi

    # Clean up temporary build directory
    rm -rf "$BUILD_DIR"
    
    # Clean up src/bin
    if [ -d "src/bin" ]; then
        rm -rf src/bin
    fi

    echo -e "\n${GREEN}========================================${NC}"
    echo -e "${GREEN}Build completed!${NC}"
    echo -e "${GREEN}Output file: build/$OUTPUT_NAME${NC}"
    echo -e "${GREEN}========================================${NC}"
    
    # Export for summary
    LAST_OUTPUT_NAME="$OUTPUT_NAME"
    LAST_VERSION_CODE="$VERSION_CODE"
    
    return 0
}

# Create build directory if it doesn't exist
if [ ! -d "build" ]; then
    mkdir build
fi

# Generate changelog once for all builds
generate_changelog "build/changelog.md"

# Build for each specified type
SUCCESS_COUNT=0
FAIL_COUNT=0
LAST_OUTPUT_NAME=""
LAST_VERSION_CODE=""

for BUILD_TYPE in "${BUILD_TYPES[@]}"; do
    build_for_type "$BUILD_TYPE"
    if [ $? -eq 0 ]; then
        ((SUCCESS_COUNT++))
    else
        ((FAIL_COUNT++))
        echo -e "${RED}Failed to build $BUILD_TYPE version${NC}"
    fi
done

# Print summary
echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}Build Summary${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Version: $VERSION${NC}"
echo -e "${GREEN}Total builds: ${#BUILD_TYPES[@]}${NC}"
echo -e "${GREEN}Successful: $SUCCESS_COUNT${NC}"
if [ $FAIL_COUNT -gt 0 ]; then
    echo -e "${RED}Failed: $FAIL_COUNT${NC}"
fi
echo -e "${GREEN}----------------------------------------${NC}"
echo -e "${GREEN}Generated files:${NC}"
echo -e "${GREEN}  - Changelog: build/changelog.md${NC}"
# Check if release was built
for BUILD_TYPE in "${BUILD_TYPES[@]}"; do
    if [ "$BUILD_TYPE" = "release" ]; then
        echo -e "${GREEN}  - Main update JSON: build/update.json${NC}"
        break
    fi
done
echo -e "${GREEN}  - ZIP packages: build/*.zip${NC}"
echo -e "${GREEN}========================================${NC}"
echo -e "${YELLOW}Note: Update JSON URL in module.prop: $UPDATE_JSON_URL${NC}"
echo -e "${YELLOW}Make sure to upload update.json to the correct location!${NC}"
echo -e "${GREEN}========================================${NC}"

# Exit with error if any build failed
if [ $FAIL_COUNT -gt 0 ]; then
    exit 1
fi

exit 0