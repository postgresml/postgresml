
export const numberToCompact = (num)  => {
  if (num >= 1e12) {
      return (num / 1e12).toFixed(1) + 'T'; // Trillion
  } else if (num >= 1e9) {
      return (num / 1e9).toFixed(1) + 'B'; // Billion
  } else if (num >= 1e6) {
      return (num / 1e6).toFixed(1) + 'M'; // Million
  } else if (num >= 1e3) {
      return (num / 1e3).toFixed(1) + 'K'; // Thousand
  } else {
      return num.toString(); // Less than a thousand
  }
};

export const compactToNumber = (compact) => {
  const suffixes = { 'K': 1e3, 'M': 1e6, 'B': 1e9, 'T': 1e12 };
  const regex = /^(\d+(\.\d+)?)([KMBT])$/;

  const match = compact.match(regex);
  if (match) {
      const number = parseFloat(match[1]);
      const suffix = match[3].toUpperCase();
      return number * suffixes[suffix];
  } else {
      return parseFloat(compact); // For numbers without suffixes
  }
};