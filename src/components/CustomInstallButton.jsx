import React, { useEffect, useRef } from 'react';
import styles from './CustomInstallButton.module.css';

const CustomInstallButton = ({ onClick, children, icon, disabled, emphasized = false, ...props }) => {
  const buttonRef = useRef(null);
  
  // Set random particle positions for the button effect
  useEffect(() => {
    if (buttonRef.current) {
      const button = buttonRef.current;
      
      // Set random positions for particles
      const setRandomPositions = () => {
        // Random positions for 5 particles
        for (let i = 1; i <= 5; i++) {
          const x = Math.floor(Math.random() * 60) - 30;
          const y = Math.floor(Math.random() * 60) - 30;
          button.style.setProperty(`--x${i}`, x);
          button.style.setProperty(`--y${i}`, y);
        }
      };
      
      // Initial setup
      setRandomPositions();
      
      // Update positions on hover for more dynamic effect
      const updatePositions = () => setRandomPositions();
      button.addEventListener('mouseenter', updatePositions);
      
      return () => {
        button.removeEventListener('mouseenter', updatePositions);
      };
    }
  }, []);

  return (
    <div style={{ position: 'relative', display: 'inline-block', color: 'yellow' }}>
      <button 
        ref={buttonRef}
        className={`${styles.button} ${emphasized ? styles.pulse : ''}`}
        onClick={onClick}
        disabled={disabled}
        {...props} // Spread any other standard button props
      >
        {icon && <span className={styles.iconWrapper}>{icon}</span>}
        <span>{children}</span>
      </button>
      <div className={styles.disclaimer}>
        Yep, I was nerd-sniped by codepen and<br />worked way too long on this button.
      </div>
    </div>
  );
};

export default CustomInstallButton; 