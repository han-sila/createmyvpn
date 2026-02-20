import { Check, Loader2 } from "lucide-react";
import type { ProgressEvent } from "../lib/types";

interface Props {
  steps: ProgressEvent[];
  currentStep: number;
}

function ProgressStepper({ steps, currentStep }: Props) {
  return (
    <div className="space-y-3">
      {steps.map((step, index) => {
        const isComplete = index < currentStep - 1;
        const isCurrent = index === currentStep - 1;
        const isError = step.status === "error";

        return (
          <div key={index} className="flex items-center gap-3">
            <div
              className={`w-7 h-7 rounded-full flex items-center justify-center flex-shrink-0 ${
                isError
                  ? "bg-red-500/20 text-red-400"
                  : isComplete
                    ? "bg-green-500/20 text-green-400"
                    : isCurrent
                      ? "bg-primary-500/20 text-primary-400"
                      : "bg-gray-800 text-gray-600"
              }`}
            >
              {isComplete ? (
                <Check className="w-4 h-4 animate-scale-in" />
              ) : isCurrent ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <span className="text-xs">{index + 1}</span>
              )}
            </div>
            <span
              className={`text-sm ${
                isError
                  ? "text-red-400"
                  : isComplete
                    ? "text-gray-400"
                    : isCurrent
                      ? "text-white"
                      : "text-gray-600"
              }`}
            >
              {step.message}
            </span>
          </div>
        );
      })}
    </div>
  );
}

export default ProgressStepper;
