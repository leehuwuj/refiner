import { cn } from "@/lib/utils";

function Skeleton({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div className={cn("skeleton", className)} {...props} />;
}

function SkeletonText({ lines = 3 }: { lines?: number }) {
  const widths = ["w-4/5", "w-3/5", "w-2/4", "w-3/4", "w-1/2"];
  return (
    <div className="space-y-2.5 pt-1">
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          className={cn("h-3", widths[i % widths.length])}
        />
      ))}
    </div>
  );
}

export { Skeleton, SkeletonText };
