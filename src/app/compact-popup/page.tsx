import dynamic from 'next/dynamic'
import { Providers } from '../provider';

const CompactPopupClient = dynamic(
    () => import('./compact-popup-client'),
);

export default function CompactPopup() {
    return (
        <Providers>
            <CompactPopupClient />
        </Providers>
    );
} 