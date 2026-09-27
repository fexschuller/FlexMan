interface InfoBadgeProps {
  text: string;
}

// Uses Pico's built-in data-tooltip attribute, so no extra CSS/JS is needed.
function InfoBadge({ text }: InfoBadgeProps) {
  return (
    <span
      data-tooltip={text}
      data-placement="right"
      tabIndex={0}
      aria-label={text}
      style={{ cursor: "help" }}
    >
      ⓘ
    </span>
  );
}

export default InfoBadge;
