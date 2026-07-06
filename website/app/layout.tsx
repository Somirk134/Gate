import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "Gate - Visual TCP Tunneling for Self-hosted Teams",
    template: "%s | Gate",
  },
  description:
    "Gate is a modern, lightweight, visual, open-source and self-hosted TCP tunneling tool built with Rust and Tauri.",
  keywords: [
    "Gate",
    "TCP tunnel",
    "NAT traversal",
    "Tauri",
    "Rust",
    "self-hosted",
    "open source",
    "developer tool",
  ],
  authors: [{ name: "Gate Contributors" }],
  creator: "Gate Contributors",
  openGraph: {
    title: "Gate - Visual TCP Tunneling",
    description:
      "A modern desktop-first TCP tunneling experience for developers, makers and self-hosted teams.",
    siteName: "Gate",
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Gate - Visual TCP Tunneling",
    description:
      "Modern, lightweight, visual, open-source and self-hosted TCP tunneling.",
  },
  icons: {
    icon: "/gate-mark.svg",
    apple: "/gate-mark.svg",
  },
};

export const viewport: Viewport = {
  colorScheme: "dark",
  themeColor: "#050608",
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body>{children}</body>
    </html>
  );
}
