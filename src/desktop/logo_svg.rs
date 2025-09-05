//! HiveTechs Logo as SVG - Matching the actual geometric bee design

/// Get the HiveTechs logo as an SVG string
pub fn get_hive_logo_svg() -> &'static str {
    r###"<svg width="64" height="64" viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
        <!-- Dark background matching logo -->
        <rect width="64" height="64" rx="8" fill="#2A2A2A"/>

        <!-- Geometric bee design in light gray/white -->
        <g transform="translate(32, 32)" stroke="#E0E0E0" stroke-width="1.2" fill="none">
            <!-- Antennae -->
            <line x1="-3" y1="-18" x2="-6" y2="-22"/>
            <line x1="3" y1="-18" x2="6" y2="-22"/>

            <!-- Head segments -->
            <path d="M -8 -18 L -8 -14 L -4 -12 L 0 -12 L 4 -12 L 8 -14 L 8 -18 L 4 -20 L 0 -20 L -4 -20 Z"/>
            <line x1="-4" y1="-20" x2="-4" y2="-12"/>
            <line x1="4" y1="-20" x2="4" y2="-12"/>

            <!-- Wings - geometric pattern -->
            <g id="left-wing">
                <path d="M -10 -10 L -22 -14 L -26 -10 L -26 -2 L -22 2 L -14 2 L -10 -2 Z"/>
                <line x1="-14" y1="-10" x2="-18" y2="-6"/>
                <line x1="-18" y1="-10" x2="-22" y2="-6"/>
                <line x1="-14" y1="-6" x2="-18" y2="-2"/>
                <line x1="-18" y1="-6" x2="-22" y2="-2"/>
            </g>

            <g id="right-wing">
                <path d="M 10 -10 L 22 -14 L 26 -10 L 26 -2 L 22 2 L 14 2 L 10 -2 Z"/>
                <line x1="14" y1="-10" x2="18" y2="-6"/>
                <line x1="18" y1="-10" x2="22" y2="-6"/>
                <line x1="14" y1="-6" x2="18" y2="-2"/>
                <line x1="18" y1="-6" x2="22" y2="-2"/>
            </g>

            <!-- Thorax segments -->
            <path d="M -8 -12 L -10 -6 L -10 -2 L -8 2 L -4 4 L 0 4 L 4 4 L 8 2 L 10 -2 L 10 -6 L 8 -12"/>
            <line x1="-8" y1="-8" x2="8" y2="-8"/>
            <line x1="-8" y1="-4" x2="8" y2="-4"/>
            <line x1="-8" y1="0" x2="8" y2="0"/>

            <!-- Abdomen segments -->
            <path d="M -8 2 L -10 8 L -8 14 L -6 18 L -2 20 L 0 20 L 2 20 L 6 18 L 8 14 L 10 8 L 8 2"/>
            <line x1="-8" y1="6" x2="8" y2="6"/>
            <line x1="-8" y1="10" x2="8" y2="10"/>
            <line x1="-6" y1="14" x2="6" y2="14"/>

            <!-- Abdomen point -->
            <path d="M -2 20 L 0 24 L 2 20"/>
        </g>
    </svg>"###
}
