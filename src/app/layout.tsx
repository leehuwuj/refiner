import "./globals.css";
import { Inter } from "next/font/google";
import { Providers } from "./provider";

const inter = Inter({ subsets: ["latin"] });

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="rounded-extra">
      <body className={inter.className}>
        <div>
          <Providers>{children}</Providers>
        </div>
      </body>
    </html>
  );
}
